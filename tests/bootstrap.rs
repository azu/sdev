#![cfg(unix)]

use std::{
    collections::BTreeMap,
    env, fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    process::{Command, Output},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

struct Fixture {
    root: PathBuf,
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    stop: Arc<AtomicBool>,
    server: Option<JoinHandle<()>>,
}
impl Fixture {
    fn new(damaged: bool) -> Self {
        Self::with_binary(damaged, fs::read(env!("CARGO_BIN_EXE_sdev")).unwrap())
    }
    fn with_binary(damaged: bool, binary: Vec<u8>) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = env::temp_dir().join(format!(
            "sdev bootstrap {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("binary"), &binary).unwrap();
        let hash = Command::new("shasum")
            .args(["-a", "256"])
            .arg(root.join("binary"))
            .output()
            .unwrap();
        assert!(hash.status.success());
        let hash = String::from_utf8(hash.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}/releases", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let version = env!("CARGO_PKG_VERSION");
        let mut files = BTreeMap::new();
        for platform in ["macos", "linux"] {
            for arch in ["arm64", "x64"] {
                let asset = format!("sdev-v{version}-{platform}-{arch}");
                files.insert(
                    format!("/releases/download/v{version}/{asset}"),
                    binary.clone(),
                );
                files.insert(
                    format!("/releases/download/v{version}/{asset}.sha256"),
                    format!(
                        "{}  {asset}\n",
                        if damaged {
                            "0".repeat(64)
                        } else {
                            hash.clone()
                        }
                    )
                    .into_bytes(),
                );
            }
        }
        let asset = format!("sdev-v{version}-windows-x64.exe");
        files.insert(
            format!("/releases/download/v{version}/{asset}.sha256"),
            format!("{hash}  {asset}\n").into_bytes(),
        );
        files.insert(
            format!("/releases/download/v{version}/attestation.sigstore.json"),
            b"test verifier fixture".to_vec(),
        );
        files.insert(
            format!("/releases/download/v{version}/sdev-release.txt"),
            format!("version={version}\nsource_commit={}\n", "a".repeat(40)).into_bytes(),
        );
        let files = Arc::new(Mutex::new(files));
        let server_files = Arc::clone(&files);
        let server_stop = Arc::clone(&stop);
        let server_requests = Arc::clone(&requests);
        let redirect = format!("{base}/tag/v{version}");
        let server = thread::spawn(move || {
            while !server_stop.load(Ordering::Relaxed) {
                let (mut socket, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap().to_owned();
                server_requests.lock().unwrap().push(path.clone());
                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }
                }
                let (status, body, headers) = if path == "/releases/latest" {
                    ("302 Found", Vec::new(), format!("Location: {redirect}\r\n"))
                } else if path == format!("/releases/tag/v{version}") {
                    ("200 OK", Vec::new(), String::new())
                } else if let Some(body) = server_files.lock().unwrap().get(&path) {
                    ("200 OK", body.clone(), String::new())
                } else {
                    ("404 Not Found", b"missing".to_vec(), String::new())
                };
                let header = format!(
                    "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                socket.write_all(header.as_bytes()).unwrap();
                let _ = socket.write_all(&body);
            }
        });
        fs::write(
            root.join("install.sh"),
            // Exercise downloads through the local HTTP fixture; production restricts both
            // the initial connection and redirects to HTTPS.
            include_str!("../install.sh"),
        )
        .unwrap();
        Self {
            root,
            base,
            requests,
            files,
            stop,
            server: Some(server),
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new("sh");
        self.update_environment(&mut command);
        command
            .arg(self.root.join("install.sh"))
            .current_dir(&self.root)
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1");
        command
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn launcher(&self, path: &str) -> PathBuf {
        let project = self.canonical_launcher(path);
        let script = project.join(path);
        let body = fs::read_to_string(&script)
            .unwrap()
            .replace("https://github.com/azu/sdev/releases", &self.base)
            .replace("--proto '=https' --proto-redir '=https' --tlsv1.2 ", "");
        fs::write(script, body).unwrap();
        project
    }
    fn canonical_launcher(&self, path: &str) -> PathBuf {
        let project = self.root.join("project");
        fs::create_dir(&project).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
        self.update_environment(&mut command);
        let result = command
            .args(["init", "--path", path])
            .current_dir(&project)
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        self.requests.lock().unwrap().clear();
        project
    }
    fn old_launcher(&self, path: &str) -> PathBuf {
        let project = self.canonical_launcher(path);
        let body = fs::read_to_string(project.join(path))
            .unwrap()
            .replace(env!("CARGO_PKG_VERSION"), "0.2.0");
        // Model a legacy launcher without checksum pins, not a new-format launcher
        // whose table points to a different fixture executable.
        let start = body.find("\nsdev_pinned_checksums() {\n").unwrap();
        let end = body[start..].find("sdev_launch() {").unwrap() + start;
        let body = format!("{}{}", &body[..start], &body[end..]);
        let body = body
            .lines()
            .filter(|line| {
                !line
                    .trim_start()
                    .starts_with("sdev_hash=$(sdev_pinned_checksums")
            })
            .map(|line| {
                if (line.trim_start().starts_with("0) main")
                    || line.trim_start().starts_with("1) main"))
                    && let Some(start) = line.find(" --sha256 ")
                {
                    let end = line[start..]
                        .find(" --verify")
                        .map_or_else(|| line[start..].find(" ;;").unwrap() + start, |n| n + start);
                    format!("{}{}", &line[..start], &line[end..])
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(project.join(path), &body).unwrap();
        fs::write(self.root.join("old-launcher"), body).unwrap();
        let binary = b"#!/bin/sh\ncase \"$1\" in\n--version) printf 'sdev 0.2.0\\n' ;;\ninit) shift; [ \"$1\" = --path ] || exit 1; mkdir -p \"$(dirname -- \"$2\")\"; cp \"$TEST_OLD_LAUNCHER\" \"$2\"; chmod +x \"$2\" ;;\n*) exec \"$TEST_CURRENT_BIN\" \"$@\" ;;\nesac\n";
        self.publish("0.2.0", binary);
        project
    }
    fn publish(&self, version: &str, bytes: &[u8]) {
        let source = self.root.join("release-source");
        fs::write(&source, bytes).unwrap();
        let hash = Command::new("shasum")
            .args(["-a", "256"])
            .arg(source)
            .output()
            .unwrap();
        assert!(hash.status.success());
        let digest = String::from_utf8(hash.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
        let mut files = self.files.lock().unwrap();
        let asset = format!("sdev-v{version}-windows-x64.exe");
        files.insert(
            format!("/releases/download/v{version}/{asset}.sha256"),
            format!("{digest}  {asset}\n").into_bytes(),
        );
        for platform in ["macos", "linux"] {
            for arch in ["arm64", "x64"] {
                let asset = format!("sdev-v{version}-{platform}-{arch}");
                files.insert(
                    format!("/releases/download/v{version}/{asset}"),
                    bytes.to_vec(),
                );
                files.insert(
                    format!("/releases/download/v{version}/{asset}.sha256"),
                    format!("{digest}  {asset}\n").into_bytes(),
                );
            }
        }
        files.insert(
            format!("/releases/download/v{version}/attestation.sigstore.json"),
            b"test verifier fixture".to_vec(),
        );
        files.insert(
            format!("/releases/download/v{version}/sdev-release.txt"),
            format!("version={version}\nsource_commit={}\n", "a".repeat(40)).into_bytes(),
        );
    }
    fn update_environment(&self, command: &mut Command) {
        let tools = self.root.join("tools");
        fs::create_dir_all(&tools).unwrap();
        let curl = Command::new("sh")
            .args(["-c", "command -v curl"])
            .output()
            .unwrap();
        assert!(curl.status.success());
        let real_curl = String::from_utf8(curl.stdout).unwrap();
        let wrapper = tools.join("curl");
        fs::write(&wrapper, "#!/bin/sh\nset -eu\nremaining=$#\nwhile [ \"$remaining\" -gt 0 ]; do\n arg=$1; shift; remaining=$((remaining-1))\n case \"$arg\" in\n  --proto|--proto-redir) shift; remaining=$((remaining-1)) ;;\n  --tlsv1.2) ;;\n  https://github.com/azu/sdev/releases*) set -- \"$@\" \"$TEST_RELEASES${arg#https://github.com/azu/sdev/releases}\" ;;\n  *) set -- \"$@\" \"$arg\" ;;\n esac\ndone\nresult=$(\"$TEST_REAL_CURL\" \"$@\") || exit $?\nprintf '%s' \"$result\" | sed \"s#$TEST_RELEASES#https://github.com/azu/sdev/releases#g\"\n").unwrap();
        fs::set_permissions(wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        command
            .env(
                "PATH",
                env::join_paths(
                    std::iter::once(tools).chain(env::split_paths(&env::var_os("PATH").unwrap())),
                )
                .unwrap(),
            )
            .env("TEST_RELEASES", &self.base)
            .env("TEST_REAL_CURL", real_curl.trim())
            .env("TEST_OLD_LAUNCHER", self.root.join("old-launcher"))
            .env("TEST_CURRENT_BIN", env!("CARGO_BIN_EXE_sdev"))
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .env_remove("SDEV_VERIFY")
            .env_remove("SDEV_LAUNCHER_PATH")
            .env_remove("SDEV_INIT_CHECKSUMS");
    }
    fn verifier(&self, version: &str, success: bool) -> Command {
        let tools = self.root.join("tools");
        fs::create_dir_all(&tools).unwrap();
        let gh = tools.join("gh");
        fs::write(&gh, "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'gh version %s\\n' \"$TEST_GH_VERSION\"; exit 0; fi\nprintf '%s\\n' \"$@\" >> \"$TEST_GH_LOG\"\nexit \"$TEST_GH_STATUS\"\n").unwrap();
        fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
        let mut command = self.command();
        command
            .env(
                "PATH",
                env::join_paths(
                    std::iter::once(tools).chain(env::split_paths(&env::var_os("PATH").unwrap())),
                )
                .unwrap(),
            )
            .env("TEST_GH_VERSION", version)
            .env("TEST_GH_STATUS", if success { "0" } else { "1" })
            .env("TEST_GH_LOG", self.root.join("gh.log"));
        command
    }
    fn no_temporary_files(&self) {
        if self.root.join("bin").exists() {
            for entry in fs::read_dir(self.root.join("bin")).unwrap() {
                assert!(
                    !entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".sdev-install.")
                );
            }
        }
    }
}

fn cache_path() -> PathBuf {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    PathBuf::from(format!(
        ".sdev/bin/{}/{platform}-{arch}/sdev",
        env!("CARGO_PKG_VERSION")
    ))
}

fn setup_snapshot(project: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    ["AGENTS.md", ".gitignore", "sdev.toml"]
        .into_iter()
        .map(|name| (project.join(name), fs::read(project.join(name)).unwrap()))
        .collect()
}
fn assert_preserved(snapshot: &[(PathBuf, Vec<u8>)]) {
    for (path, bytes) in snapshot {
        assert_eq!(fs::read(path).unwrap(), *bytes, "{}", path.display());
    }
}
fn assert_update_clean(project: &std::path::Path, parent: &str) {
    assert!(!project.join(".sdev/update.lock").exists());
    for directory in [project.join(".sdev"), project.join(parent)] {
        if directory.is_dir() {
            for entry in fs::read_dir(directory).unwrap() {
                assert!(
                    !entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".sdev-update.")
                );
            }
        }
    }
}

#[test]
fn project_update_keeps_custom_path_settings_permissions_and_offline_use() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("tools/scripts/sdev");
    fs::write(
        project.join("sdev.toml"),
        "# keep these bytes\r\nlanguage = \"ja\"\r\ntranslations = [\"en\"]\r\n",
    )
    .unwrap();
    let snapshot = setup_snapshot(&project);
    let launcher = project.join("tools/scripts/sdev");
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o551)).unwrap();
    let mut command = Command::new(&launcher);
    command.arg("update").current_dir(&project);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(
        report.contains("0.2.0")
            && report.contains(env!("CARGO_PKG_VERSION"))
            && report.contains("tools/scripts/sdev")
    );
    assert_preserved(&snapshot);
    assert_eq!(
        fs::metadata(&launcher).unwrap().permissions().mode() & 0o777,
        0o551
    );
    assert!(project.join(cache_path()).is_file());
    assert_update_clean(&project, "tools/scripts");
    let output = Command::new(&launcher)
        .arg("--version")
        .current_dir(&project)
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("sdev {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn reinstall_updates_a_genuine_old_launcher_and_same_version_is_unchanged() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let snapshot = setup_snapshot(&project);
    let mut command = fixture.command();
    command
        .current_dir(&project)
        .args(["--version", env!("CARGO_PKG_VERSION"), "--init"]);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_preserved(&snapshot);
    let launcher = project.join("bin/sdev");
    let body = fs::read(&launcher).unwrap();
    assert!(
        String::from_utf8_lossy(&body)
            .contains(&format!("generated by sdev {}", env!("CARGO_PKG_VERSION")))
    );
    let before = fs::metadata(&launcher).unwrap().modified().unwrap();
    let mut command = fixture.command();
    command
        .current_dir(&project)
        .args(["--version", env!("CARGO_PKG_VERSION")]);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("already selected")
    );
    assert_eq!(fs::read(&launcher).unwrap(), body);
    assert_eq!(fs::metadata(&launcher).unwrap().modified().unwrap(), before);
    assert_preserved(&snapshot);
    assert_update_clean(&project, "bin");
}

#[test]
fn updates_reject_edited_launchers_before_acquiring_the_new_version() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let launcher = project.join("bin/sdev");
    let mut body = fs::read(&launcher).unwrap();
    body.extend_from_slice(b"\n# user edit\n");
    fs::write(&launcher, &body).unwrap();
    let snapshot = setup_snapshot(&project);
    let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
    command
        .current_dir(&project)
        .args(["update", "--version", env!("CARGO_PKG_VERSION")]);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Launcher has been edited"));
    assert_eq!(fs::read(launcher).unwrap(), body);
    assert_preserved(&snapshot);
    assert!(
        !fixture
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|path| path.contains(&format!("/download/v{}/", env!("CARGO_PKG_VERSION"))))
    );
    assert_update_clean(&project, "bin");
}

#[test]
fn a_damaged_update_preserves_the_old_launcher_without_publishing_a_new_cache() {
    let fixture = Fixture::new(true);
    let project = fixture.old_launcher("bin/sdev");
    let body = fs::read(project.join("bin/sdev")).unwrap();
    let snapshot = setup_snapshot(&project);
    let mut command = fixture.command();
    command
        .current_dir(&project)
        .args(["--version", env!("CARGO_PKG_VERSION"), "--init"]);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("checksum mismatch"));
    assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), body);
    assert!(!project.join(cache_path()).exists());
    assert_preserved(&snapshot);
    assert_update_clean(&project, "bin");
}

#[test]
fn requested_update_attestation_never_falls_back_and_is_used_for_both_versions() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let body = fs::read(project.join("bin/sdev")).unwrap();
    for success in [false, true] {
        let environment = fixture.verifier("2.100.0", success);
        let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
        for (key, value) in environment.get_envs() {
            if let Some(value) = value {
                command.env(key, value);
            } else {
                command.env_remove(key);
            }
        }
        fixture.update_environment(&mut command);
        command
            .current_dir(&project)
            .args([
                "update",
                "--version",
                env!("CARGO_PKG_VERSION"),
                "--source-commit",
                &"a".repeat(40),
            ])
            .env("SDEV_VERIFY", "1");
        let output = command.output().unwrap();
        assert_eq!(output.status.success(), success, "{output:?}");
        if !success {
            assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), body);
            assert!(!project.join(cache_path()).exists());
        }
        assert_update_clean(&project, "bin");
    }
    let log = fs::read_to_string(fixture.root.join("gh.log")).unwrap();
    assert!(
        log.contains("sdev-v0.2.0-")
            && log.contains(&format!("sdev-v{}-", env!("CARGO_PKG_VERSION")))
    );
    assert!(
        log.contains("--source-digest\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n--signer-digest")
    );
}

#[test]
fn update_rejects_symlink_cache_and_an_existing_update_lock() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let body = fs::read(project.join("bin/sdev")).unwrap();
    let outside = fixture.root.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, project.join(".sdev")).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
    command
        .current_dir(&project)
        .args(["update", "--version", env!("CARGO_PKG_VERSION")]);
    fixture.update_environment(&mut command);
    assert!(!command.output().unwrap().status.success());
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
    fs::remove_file(project.join(".sdev")).unwrap();
    fs::create_dir_all(project.join(".sdev/update.lock")).unwrap();
    let output = command.output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Update is locked"));
    assert!(project.join(".sdev/update.lock").is_dir());
    assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), body);
    assert!(fixture.requests.lock().unwrap().is_empty());
}

#[test]
fn failure_to_generate_a_new_launcher_keeps_the_previous_entry_point() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let body = fs::read(project.join("bin/sdev")).unwrap();
    let binary = format!(
        "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'sdev {}\\n'; else exit 7; fi\n",
        env!("CARGO_PKG_VERSION")
    );
    fixture.publish(env!("CARGO_PKG_VERSION"), binary.as_bytes());
    let snapshot = setup_snapshot(&project);
    let mut command = fixture.command();
    command
        .current_dir(&project)
        .args(["--version", env!("CARGO_PKG_VERSION")]);
    fixture.update_environment(&mut command);
    let output = command.output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Could not prepare the new launcher"));
    assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), body);
    assert!(!project.join(cache_path()).exists());
    assert_preserved(&snapshot);
    assert_update_clean(&project, "bin");
}

#[test]
fn a_launcher_edited_during_download_is_not_replaced() {
    let fixture = Fixture::new(false);
    let project = fixture.old_launcher("bin/sdev");
    let body = fs::read(project.join("bin/sdev")).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
    command
        .current_dir(&project)
        .args(["update", "--version", env!("CARGO_PKG_VERSION")]);
    fixture.update_environment(&mut command);
    command.env("TEST_CHANGED_LAUNCHER", project.join("bin/sdev"));
    let wrapper = fixture.root.join("tools/curl");
    let script = fs::read_to_string(&wrapper).unwrap().replace("result=$(", &format!("case \"$arg\" in */download/v{}/*) printf '\\n# concurrent edit\\n' >> \"$TEST_CHANGED_LAUNCHER\" ;; esac\nresult=$(", env!("CARGO_PKG_VERSION")));
    fs::write(wrapper, script).unwrap();
    let output = command.output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Launcher changed during update"));
    let after = fs::read(project.join("bin/sdev")).unwrap();
    assert!(after.starts_with(&body));
    assert!(String::from_utf8_lossy(&after).contains("# concurrent edit"));
    assert!(!project.join(cache_path()).exists());
    assert_update_clean(&project, "bin");
}

#[test]
fn cloned_launcher_pins_the_version_preserves_cwd_arguments_and_exit_status_then_runs_offline() {
    let fixture = Fixture::with_binary(false, format!(
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'sdev {}'; exit 0; fi\nprintf '%s\\n' \"$PWD\" \"$@\"\nexit 7\n", env!("CARGO_PKG_VERSION")
    ).into_bytes());
    let original = fixture.launcher("tools/scripts/sdev");
    let cloned = fixture.root.join("cloned project");
    fs::rename(original, &cloned).unwrap();
    let working = cloned.join("work");
    fs::create_dir(&working).unwrap();
    let agents = fs::read(cloned.join("AGENTS.md")).unwrap();
    let script = fs::read(cloned.join("tools/scripts/sdev")).unwrap();
    let invoke = || {
        Command::new(cloned.join("tools/scripts/sdev"))
            .args(["template", "two words", "$(no-eval)", ""])
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .current_dir(&working)
            .output()
            .unwrap()
    };
    for _ in 0..2 {
        let output = invoke();
        assert_eq!(output.status.code(), Some(7), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "{}\ntemplate\ntwo words\n$(no-eval)\n\n",
                working.canonicalize().unwrap().display()
            )
        );
        assert_eq!(fs::read(cloned.join("AGENTS.md")).unwrap(), agents);
        assert_eq!(fs::read(cloned.join("tools/scripts/sdev")).unwrap(), script);
        assert!(cloned.join(cache_path()).is_file());
    }
    let requests = fixture.requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(
        requests
            .iter()
            .all(|p| p.contains(&format!("/download/v{}/", env!("CARGO_PKG_VERSION"))))
    );
}

#[test]
fn committed_hash_rejects_replaced_binary_and_sidecar_before_execution_or_update() {
    for update in [false, true] {
        let fixture = Fixture::new(false);
        let project = fixture.canonical_launcher("bin/sdev");
        let original = fs::read(project.join("bin/sdev")).unwrap();
        let malicious = format!(
            "#!/bin/sh\nprintf ran > '{}'\nprintf 'sdev {}\\n'\n",
            fixture.root.join("ran").display(),
            env!("CARGO_PKG_VERSION")
        );
        fixture.publish(env!("CARGO_PKG_VERSION"), malicious.as_bytes());
        let mut command = if update {
            let mut command = fixture.command();
            command.args(["--update", "--version", env!("CARGO_PKG_VERSION")]);
            command
        } else {
            let mut command = Command::new(project.join("bin/sdev"));
            fixture.update_environment(&mut command);
            command.arg("help");
            command
        };
        let result = command.current_dir(&project).output().unwrap();
        assert!(!result.status.success(), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("SHA-256 checksum mismatch"));
        assert!(!fixture.root.join("ran").exists());
        assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), original);
        assert!(!project.join(cache_path()).exists());
        assert!(
            fixture
                .requests
                .lock()
                .unwrap()
                .iter()
                .all(|p| !p.ends_with(".sha256"))
        );
    }
}

#[test]
fn missing_checksum_for_another_platform_preserves_project_during_init_and_update() {
    for update in [false, true] {
        let fixture = Fixture::new(false);
        let project = if update {
            fixture.old_launcher("bin/sdev")
        } else {
            let project = fixture.root.join("project");
            fs::create_dir(&project).unwrap();
            project
        };
        let original = fs::read(project.join("bin/sdev")).ok();
        fixture.files.lock().unwrap().remove(&format!(
            "/releases/download/v{}/sdev-v{}-windows-x64.exe.sha256",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION")
        ));
        let mut command = fixture.command();
        command
            .current_dir(&project)
            .args(["--version", env!("CARGO_PKG_VERSION")]);
        if update {
            command.arg("--update");
        } else {
            command.arg("--init");
        }
        let result = command.output().unwrap();
        assert!(!result.status.success(), "{result:?}");
        if update {
            assert_eq!(fs::read(project.join("bin/sdev")).ok(), original);
        } else {
            assert!(!project.join("AGENTS.md").exists());
            assert!(!project.join("sdev.toml").exists());
        }
    }
}

#[test]
fn direct_initialization_requires_a_complete_valid_remote_checksum_set_before_writing() {
    for missing in [false, true] {
        let fixture = Fixture::new(false);
        let project = fixture.root.join("direct-init");
        fs::create_dir(&project).unwrap();
        fs::write(project.join("AGENTS.md"), "Keep these instructions.\n").unwrap();
        fs::write(project.join(".gitignore"), "/keep\n").unwrap();
        let checksum = format!(
            "/releases/download/v{}/sdev-v{}-windows-x64.exe.sha256",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION")
        );
        if missing {
            fixture.files.lock().unwrap().remove(&checksum);
        } else {
            fixture
                .files
                .lock()
                .unwrap()
                .insert(checksum, b"invalid checksum\n".to_vec());
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
        fixture.update_environment(&mut command);
        let result = command.arg("init").current_dir(&project).output().unwrap();
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(
            fs::read_to_string(project.join("AGENTS.md")).unwrap(),
            "Keep these instructions.\n"
        );
        assert_eq!(
            fs::read_to_string(project.join(".gitignore")).unwrap(),
            "/keep\n"
        );
        assert_eq!(fs::read_dir(&project).unwrap().count(), 2);
    }
}

#[test]
fn the_same_committed_launcher_selects_each_supported_unix_platform() {
    let fixture = Fixture::with_binary(
        false,
        format!(
            "#!/bin/sh\nprintf 'sdev {}\\n'\n",
            env!("CARGO_PKG_VERSION")
        )
        .into_bytes(),
    );
    let project = fixture.launcher("bin/sdev");
    let uname = fixture.root.join("tools/uname");
    fs::write(&uname, "#!/bin/sh\ncase \"$1\" in -s) echo \"$TEST_OS\" ;; -m) echo \"$TEST_ARCH\" ;; *) exit 1 ;; esac\n").unwrap();
    fs::set_permissions(&uname, fs::Permissions::from_mode(0o755)).unwrap();
    for (os, arch, label) in [
        ("Darwin", "arm64", "macos-arm64"),
        ("Darwin", "x86_64", "macos-x64"),
        ("Linux", "aarch64", "linux-arm64"),
        ("Linux", "x86_64", "linux-x64"),
    ] {
        let mut command = Command::new(project.join("bin/sdev"));
        fixture.update_environment(&mut command);
        let result = command
            .arg("--version")
            .env("TEST_OS", os)
            .env("TEST_ARCH", arch)
            .current_dir(&project)
            .output()
            .unwrap();
        assert!(result.status.success(), "{label}: {result:?}");
        let requests = fixture.requests.lock().unwrap();
        assert_eq!(
            requests.last().unwrap(),
            &format!(
                "/releases/download/v{}/sdev-v{}-{label}",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_VERSION")
            )
        );
        assert!(requests.iter().all(|p| !p.ends_with(".sha256")));
    }
}

#[test]
fn launcher_download_failure_and_symlink_cache_never_execute_a_binary() {
    let fixture = Fixture::new(true);
    let project = fixture.launcher("sdev");
    let invoke = || {
        Command::new(project.join("sdev"))
            .arg("help")
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .current_dir(&project)
            .output()
            .unwrap()
    };
    let output = invoke();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SHA-256 checksum mismatch"));
    assert!(!project.join(cache_path()).exists());
    fs::remove_dir_all(project.join(".sdev")).unwrap();
    let outside = fixture.root.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, project.join(".sdev")).unwrap();
    let count = fixture.requests.lock().unwrap().len();
    assert!(!invoke().status.success());
    assert_eq!(fixture.requests.lock().unwrap().len(), count);
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn installer_supports_custom_directory_and_preserves_committed_launchers() {
    let fixture = Fixture::new(false);
    let output = fixture.run(&[
        "--version",
        env!("CARGO_PKG_VERSION"),
        "--install-dir",
        "custom/bin",
    ]);
    assert!(output.status.success(), "{output:?}");
    assert!(fixture.root.join("custom/bin/sdev").is_file());
    let project = fixture.launcher("bin/sdev");
    let script = fs::read(project.join("bin/sdev")).unwrap();
    let output = fixture
        .command()
        .current_dir(&project)
        .args(["--version", env!("CARGO_PKG_VERSION")])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(project.join("bin/sdev")).unwrap(), script);
}

#[test]
fn launcher_requested_attestation_failure_never_falls_back() {
    let fixture = Fixture::new(false);
    let project = fixture.launcher("bin/sdev");
    let verifier = fixture.verifier("2.93.0", false);
    let mut command = Command::new(project.join("bin/sdev"));
    for (key, value) in verifier.get_envs() {
        if let Some(value) = value {
            command.env(key, value);
        } else {
            command.env_remove(key);
        }
    }
    let output = command
        .env("SDEV_VERIFY", "1")
        .arg("help")
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Attestation verification failed"));
    assert!(!project.join(cache_path()).exists());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("provenance NOT verified"));
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.server.take().unwrap().join().unwrap();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn latest_install_uses_public_redirects_and_can_initialize_then_restore_without_editing_instructions()
 {
    let fixture = Fixture::new(false);
    let output = fixture.run(&["--init"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("provenance NOT verified"));
    assert!(fixture.root.join("bin/sdev").is_file());
    let agents = fs::read(fixture.root.join("AGENTS.md")).unwrap();
    assert!(String::from_utf8_lossy(&agents).contains("./bin/sdev help"));
    let ignores = fs::read(fixture.root.join(".gitignore")).unwrap();
    assert!(
        fixture
            .requests
            .lock()
            .unwrap()
            .contains(&"/releases/latest".to_owned())
    );
    assert!(
        fixture
            .requests
            .lock()
            .unwrap()
            .iter()
            .all(|p| !p.contains("api.github"))
    );
    fs::remove_file(fixture.root.join("bin/sdev")).unwrap();
    assert!(
        fixture
            .run(&["--version", env!("CARGO_PKG_VERSION")])
            .status
            .success()
    );
    assert_eq!(fs::read(fixture.root.join("AGENTS.md")).unwrap(), agents);
    assert_eq!(fs::read(fixture.root.join(".gitignore")).unwrap(), ignores);
    fixture.no_temporary_files();
}

#[test]
fn failed_download_or_checksum_preserves_the_old_executable_without_initializing() {
    for damaged in [true, false] {
        let fixture = Fixture::with_binary(
            damaged,
            b"#!/bin/sh\nprintf ran > binary-ran\nexit 0\n".to_vec(),
        );
        fs::create_dir(fixture.root.join("bin")).unwrap();
        fs::write(fixture.root.join("bin/sdev"), "old binary").unwrap();
        let version = if damaged {
            env!("CARGO_PKG_VERSION")
        } else {
            "9.9.9"
        };
        let output = fixture.run(&["--version", version, "--init"]);
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.root.join("bin/sdev")).unwrap(),
            "old binary"
        );
        assert!(!fixture.root.join("AGENTS.md").exists());
        assert!(!fixture.root.join("binary-ran").exists());
        fixture.no_temporary_files();
    }
}

#[test]
fn attestation_failure_never_falls_back_or_runs_the_download_and_old_gh_never_downloads() {
    for old in [false, true] {
        let fixture = Fixture::with_binary(
            false,
            b"#!/bin/sh\nprintf ran > binary-ran\nexit 0\n".to_vec(),
        );
        fs::create_dir(fixture.root.join("bin")).unwrap();
        fs::write(fixture.root.join("bin/sdev"), "old binary").unwrap();
        let output = fixture
            .verifier(if old { "2.92.0" } else { "2.100.0" }, false)
            .args(["--version", env!("CARGO_PKG_VERSION"), "--verify", "--init"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.root.join("bin/sdev")).unwrap(),
            "old binary"
        );
        assert!(!fixture.root.join("AGENTS.md").exists());
        assert!(!fixture.root.join("binary-ran").exists());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("provenance NOT verified"));
        if old {
            assert!(fixture.requests.lock().unwrap().is_empty());
        }
        fixture.no_temporary_files();
    }
}

#[test]
fn invalid_downloaded_programs_do_not_replace_the_old_executable() {
    for body in ["#!/bin/sh\nexit 9\n", "#!/bin/sh\nprintf 'sdev 9.9.9\\n'\n"] {
        let fixture = Fixture::with_binary(false, body.as_bytes().to_vec());
        fs::create_dir(fixture.root.join("bin")).unwrap();
        fs::write(fixture.root.join("bin/sdev"), "old binary").unwrap();
        let output = fixture.run(&["--version", env!("CARGO_PKG_VERSION"), "--init"]);
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.root.join("bin/sdev")).unwrap(),
            "old binary"
        );
        assert!(!fixture.root.join("AGENTS.md").exists());
        fixture.no_temporary_files();
    }
}

#[test]
fn a_truncated_piped_installer_does_not_start_the_installation() {
    let fixture = Fixture::new(false);
    let script = fs::read_to_string(fixture.root.join("install.sh")).unwrap();
    let truncated = script.split("\ncheck_verifier()").next().unwrap();
    fs::write(fixture.root.join("install.sh"), truncated).unwrap();
    fixture.run(&[]);
    assert!(fixture.requests.lock().unwrap().is_empty());
    assert!(!fixture.root.join("bin").exists());
    assert!(!fixture.root.join("AGENTS.md").exists());
}

#[test]
fn verification_uses_a_local_bundle_and_binds_both_commit_digests_to_the_signed_manifest() {
    let fixture = Fixture::new(false);
    let output = fixture
        .verifier("2.100.0", true)
        .args([
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--verify",
            "--source-commit",
            &"a".repeat(40),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fs::read_to_string(fixture.root.join("gh.log")).unwrap();
    for required in [
        "--bundle",
        "--hostname\ngithub.com",
        "--repo\nazu/sdev",
        "--signer-workflow\nazu/sdev/.github/workflows/build.yml",
        "--deny-self-hosted-runners",
        "--source-ref\nrefs/heads/main",
    ] {
        assert!(log.contains(required), "{required}");
    }
    assert_eq!(log.matches("--source-digest").count(), 2);
    assert_eq!(log.matches("--signer-digest").count(), 2);
    assert!(log.contains(&"a".repeat(40)));
    assert!(fixture.root.join("bin/sdev").is_file());
    assert!(!fixture.root.join("AGENTS.md").exists());
    fixture.no_temporary_files();
}

#[test]
fn a_valid_attestation_for_a_different_pinned_commit_does_not_run_or_replace_the_binary() {
    let fixture = Fixture::with_binary(
        false,
        b"#!/bin/sh\nprintf ran > binary-ran\nexit 0\n".to_vec(),
    );
    fs::create_dir(fixture.root.join("bin")).unwrap();
    fs::write(fixture.root.join("bin/sdev"), "old binary").unwrap();
    let output = fixture
        .verifier("2.100.0", true)
        .args([
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--verify",
            "--source-commit",
            &"b".repeat(40),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not match the pinned commit"));
    assert_eq!(
        fs::read_to_string(fixture.root.join("bin/sdev")).unwrap(),
        "old binary"
    );
    assert!(!fixture.root.join("binary-ran").exists());
    fixture.no_temporary_files();
}

#[test]
fn invalid_arguments_and_unsafe_destinations_fail_without_downloading() {
    let fixture = Fixture::new(false);
    for version in ["v0.1.0", "1..0", "1.2.3.4", "01.2.3", "../1.2.3"] {
        assert!(
            !fixture.run(&["--version", version]).status.success(),
            "{version}"
        );
    }
    assert!(fixture.requests.lock().unwrap().is_empty());
    fs::create_dir(fixture.root.join("elsewhere")).unwrap();
    symlink("elsewhere", fixture.root.join("bin")).unwrap();
    assert!(
        !fixture
            .run(&["--version", env!("CARGO_PKG_VERSION")])
            .status
            .success()
    );
    assert!(fixture.requests.lock().unwrap().is_empty());
    assert!(
        fs::read_dir(fixture.root.join("elsewhere"))
            .unwrap()
            .next()
            .is_none()
    );
}
