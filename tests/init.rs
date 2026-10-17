use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = env::temp_dir().join(format!(
            "sdev-init-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sdev"))
            .args(args)
            .current_dir(&self.0)
            .env("PATH", "")
            .output()
            .unwrap()
    }
    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.0.join(name)).unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn init_creates_only_instructions_and_ignores_and_routes_to_current_catalogs() {
    let project = Project::new();
    let output = project.run(&["init"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    let agents = project.read("AGENTS.md");
    for text in [
        "<!-- sdev:begin -->",
        "仕様の作成・変更、設計、実装、検証",
        "表示される一覧から",
        "利用先の指示・規約と既存の仕様を優先",
        "docs/products/",
        "<!-- sdev:end -->",
    ] {
        assert!(agents.contains(text), "{text}");
    }
    #[cfg(not(windows))]
    assert!(agents.contains("`./bin/sdev help`"));
    #[cfg(windows)]
    assert!(agents.contains("`./bin/sdev.exe help`"));
    assert!(!agents.contains("help product"));
    assert!(!agents.contains("template product"));
    assert_eq!(project.read(".gitignore"), "/bin/sdev\n/bin/sdev.exe\n");
    assert_eq!(fs::read_dir(&project.0).unwrap().count(), 2);
}

#[test]
fn repeated_init_preserves_existing_bytes_crlf_and_edited_sdev_instructions() {
    let project = Project::new();
    let original = "# 開発規約\r\n\r\n既存の指示を優先する。";
    fs::write(project.0.join("AGENTS.md"), original).unwrap();
    fs::write(project.0.join(".gitignore"), "/target/\r\n/bin/sdev\r\n").unwrap();
    assert!(project.run(&["init"]).status.success());
    let agents = project.read("AGENTS.md");
    assert!(agents.starts_with(original));
    assert!(agents.replace("\r\n", "").find('\n').is_none());
    let edited = agents.replace("## 仕様を起点にした開発", "## 利用者が編集した指示");
    fs::write(project.0.join("AGENTS.md"), &edited).unwrap();
    let ignore = project.read(".gitignore");
    assert_eq!(ignore.matches("/bin/sdev\r\n").count(), 1);
    let output = project.run(&["init"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("既存の設定を保持")
    );
    assert_eq!(project.read("AGENTS.md"), edited);
    assert_eq!(project.read(".gitignore"), ignore);
    assert_eq!(fs::read_dir(&project.0).unwrap().count(), 2);
}

#[test]
fn invalid_files_or_markers_are_rejected_before_changing_either_document() {
    for case in 0..7 {
        let project = Project::new();
        let agents = match case {
            0 => "<!-- sdev:begin -->",
            1 => "<!-- sdev:end -->\n<!-- sdev:begin -->",
            2 => "<!-- sdev:begin --><!-- sdev:begin --><!-- sdev:end -->",
            _ => "既存の指示",
        };
        fs::write(project.0.join("AGENTS.md"), agents).unwrap();
        match case {
            3 => fs::create_dir(project.0.join(".gitignore")).unwrap(),
            4 => fs::write(project.0.join(".gitignore"), [0xff]).unwrap(),
            5 => fs::write(project.0.join("AGENTS.md"), [0xff]).unwrap(),
            6 => {
                fs::remove_file(project.0.join("AGENTS.md")).unwrap();
                fs::create_dir(project.0.join("AGENTS.md")).unwrap();
            }
            _ => {}
        }
        let before = fs::read(project.0.join("AGENTS.md")).ok();
        let result = project.run(&["init"]);
        assert_eq!(result.status.code(), Some(1), "case {case}");
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
        assert_eq!(fs::read(project.0.join("AGENTS.md")).ok(), before);
        if !matches!(case, 3 | 4) {
            assert!(!project.0.join(".gitignore").exists());
        }
    }
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_and_existing_permissions_are_preserved() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let project = Project::new();
    fs::write(project.0.join("original"), "元の指示").unwrap();
    for name in ["AGENTS.md", ".gitignore"] {
        symlink("original", project.0.join(name)).unwrap();
        assert_eq!(project.run(&["init"]).status.code(), Some(1));
        assert_eq!(project.read("original"), "元の指示");
        fs::remove_file(project.0.join(name)).unwrap();
    }
    fs::write(project.0.join("AGENTS.md"), "既存").unwrap();
    fs::set_permissions(
        project.0.join("AGENTS.md"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    assert!(project.run(&["init"]).status.success());
    assert_eq!(
        fs::metadata(project.0.join("AGENTS.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}

#[test]
fn help_and_invalid_arguments_do_not_write_files() {
    let project = Project::new();
    for args in [
        vec!["init", "--help"],
        vec!["init", "-h"],
        vec!["init", "--force"],
        vec!["init", "."],
    ] {
        let output = project.run(&args);
        assert_eq!(
            output.status.code(),
            Some(if args[1] == "--help" || args[1] == "-h" {
                0
            } else {
                2
            })
        );
        assert_eq!(fs::read_dir(&project.0).unwrap().count(), 0);
    }
}
