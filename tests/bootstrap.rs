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
    requests: Arc<Mutex<Vec<String>>>,
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
        files.insert(
            format!("/releases/download/v{version}/attestation.sigstore.json"),
            b"test verifier fixture".to_vec(),
        );
        files.insert(
            format!("/releases/download/v{version}/sdev-release.txt"),
            format!("version={version}\nsource_commit={}\n", "a".repeat(40)).into_bytes(),
        );
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
                } else if let Some(body) = files.get(&path) {
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
            include_str!("../install.sh")
                .replace("https://github.com/azu/sdev/releases", &base)
                .replace("--proto '=https' --proto-redir '=https' --tlsv1.2 ", ""),
        )
        .unwrap();
        Self {
            root,
            requests,
            stop,
            server: Some(server),
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new("sh");
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
