use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

mod support;

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "sdev-languages-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, text: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
        command
            .args(args)
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", "")
            .env("LANG", "ja_JP.UTF-8")
            .env("LC_ALL", "ja_JP.UTF-8");
        for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        #[cfg(unix)]
        command.env("SDEV_INIT_CHECKSUMS", support::checksums());
        command.output().unwrap()
    }
    fn success(&self, args: &[&str]) -> String {
        let result = self.run(args);
        assert!(
            result.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        String::from_utf8(result.stdout).unwrap()
    }
    fn snapshot(&self) -> Vec<(PathBuf, Vec<u8>)> {
        fn visit(
            root: &std::path::Path,
            dir: &std::path::Path,
            result: &mut Vec<(PathBuf, Vec<u8>)>,
        ) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(root, &path, result);
                } else {
                    result.push((
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        fs::read(path).unwrap(),
                    ));
                }
            }
        }
        let mut result = Vec::new();
        visit(&self.0, &self.0, &mut result);
        result.sort();
        result
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn unconfigured_output_is_english_regardless_of_terminal_locale() {
    let p = Project::new();
    assert!(
        p.success(&["help"])
            .contains("Choose a guide for your task")
    );
    assert!(p.success(&["help", "requirements"]).starts_with("# "));
    assert!(
        p.success(&["template", "design"])
            .contains("## Implementation approach")
    );
    assert!(
        p.success(&["init", "--help"])
            .contains("Run at the project root")
    );
    assert!(p.snapshot().is_empty());
}

#[test]
fn project_default_and_command_overrides_select_bodies_without_writes() {
    let p = Project::new();
    p.write(
        "sdev.toml",
        "# Shared choice\r\nlanguage = 'ja'\r\ntranslations = ['en']\r\n",
    );
    let before = p.snapshot();
    let ja = p.success(&["help", "design"]);
    assert!(ja.starts_with("# Designを書く"));
    assert!(ja.contains("sdev template design --lang ja"));
    let en = p.success(&["help", "design", "--lang", "en"]);
    assert!(en.starts_with("# Writing Design"));
    assert!(en.contains("sdev template design --lang en"));
    assert_eq!(en, p.success(&["--lang=en", "help", "design"]));
    assert_eq!(ja, p.success(&["--lang", "ja", "help", "design"]));
    assert_eq!(before, p.snapshot());
}

#[test]
fn initialization_shares_languages_without_generating_specifications() {
    for (default, translation) in [("en", "ja"), ("ja", "en")] {
        let p = Project::new();
        p.success(&["init", "--lang", default, "--translations", translation]);
        let config = fs::read_to_string(p.0.join("sdev.toml")).unwrap();
        assert_eq!(
            config,
            format!("language = \"{default}\"\ntranslations = [\"{translation}\"]\n")
        );
        let agents = fs::read_to_string(p.0.join("AGENTS.md")).unwrap();
        for expected in ["sdev.toml", ".md", "help", "template"] {
            assert!(agents.contains(expected));
        }
        assert!(!p.0.join("docs").exists());
        assert!(!p.0.join(".sdev").exists());
        let before = p.snapshot();
        p.success(&["init"]);
        p.success(&["init", "--lang", default, "--translations", translation]);
        assert_eq!(before, p.snapshot());
    }
}

#[test]
fn default_initialization_uses_english_and_no_translations() {
    let p = Project::new();
    p.success(&["init"]);
    assert_eq!(
        fs::read_to_string(p.0.join("sdev.toml")).unwrap(),
        "language = \"en\"\ntranslations = []\n"
    );
    assert!(
        fs::read_to_string(p.0.join("AGENTS.md"))
            .unwrap()
            .contains("Specifications-first development")
    );
}

#[test]
fn initialization_preserves_config_bytes_and_edited_instruction_blocks() {
    let p = Project::new();
    p.write(
        "sdev.toml",
        "# Keep this comment\r\ntranslations = ['en']\r\nlanguage = 'ja'\r\n",
    );
    p.success(&["init"]);
    let agents = fs::read_to_string(p.0.join("AGENTS.md"))
        .unwrap()
        .replace("<!-- sdev:end -->", "User rule.\n<!-- sdev:end -->");
    p.write("AGENTS.md", agents);
    let before = p.snapshot();
    p.success(&["init"]);
    assert_eq!(before, p.snapshot());
    for args in [
        &["init", "--lang", "en"][..],
        &["init", "--translations", ""][..],
    ] {
        let result = p.run(args);
        assert_eq!(result.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("sdev.toml"));
        assert_eq!(before, p.snapshot());
    }
}

#[test]
fn invalid_settings_fail_before_any_initialization_writes() {
    for body in [
        "language = 'fr'",
        "language = 1",
        "translations = 'ja'",
        "translations = [true]",
        "language = 'en'\ntranslations = ['en']",
        "translations = ['ja', 'ja']",
        "unknown = true",
        "[display]\nlanguage = 'ja'",
        "language = 'en'\nlanguage = 'ja'",
        "[",
    ] {
        let p = Project::new();
        p.write("sdev.toml", body);
        let before = p.snapshot();
        for args in [
            &["init"][..],
            &["help"][..],
            &["template", "design"][..],
            &["lint", "--format", "json"][..],
        ] {
            let result = p.run(args);
            assert_eq!(result.status.code(), Some(2), "{body}: {args:?}");
            assert!(result.stdout.is_empty());
            assert!(!result.stderr.is_empty());
            assert_eq!(before, p.snapshot());
        }
        assert_eq!(
            p.success(&["--version"]),
            format!("sdev {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn invalid_language_options_do_not_write_files() {
    for args in [
        &["init", "--lang"][..],
        &["help", "--lang", "fr"],
        &["--lang", "ja", "help", "--lang=en"],
        &["init", "--lang", "en", "--translations", "en"],
        &["init", "--translations", "ja,ja"],
        &["init", "--translations", "fr"],
        &["init", "--translations", "ja,"],
        &["init", "--translations"],
        &["help", "--translations", "ja"],
        &["init", "--translations", "ja", "--translations", "ja"],
    ] {
        let p = Project::new();
        let result = p.run(args);
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
        assert!(p.snapshot().is_empty());
    }
}

#[test]
fn invalid_utf8_and_directory_settings_prevent_init() {
    let p = Project::new();
    p.write("sdev.toml", [0xff]);
    assert_eq!(p.run(&["init"]).status.code(), Some(2));
    assert!(!p.0.join("AGENTS.md").exists());
    let p = Project::new();
    fs::create_dir(p.0.join("sdev.toml")).unwrap();
    assert_eq!(p.run(&["init"]).status.code(), Some(2));
    assert!(!p.0.join(".gitignore").exists());
}

#[cfg(unix)]
#[test]
fn symlink_settings_and_reserved_launcher_paths_are_rejected() {
    let p = Project::new();
    p.write("actual.toml", "language = 'en'");
    std::os::unix::fs::symlink("actual.toml", p.0.join("sdev.toml")).unwrap();
    assert_eq!(p.run(&["init"]).status.code(), Some(2));
    assert!(!p.0.join("AGENTS.md").exists());
    let p = Project::new();
    assert_eq!(
        p.run(&["init", "--path", "sdev.toml"]).status.code(),
        Some(2)
    );
    assert!(p.snapshot().is_empty());
}

#[test]
fn configuration_is_not_inherited_from_parent_directories() {
    let p = Project::new();
    p.write("sdev.toml", "language = 'ja'");
    fs::create_dir(p.0.join("child")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sdev"))
        .arg("help")
        .current_dir(p.0.join("child"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("Choose a guide")
    );
}

#[test]
fn lint_localizes_text_but_keeps_codes_paths_and_counts() {
    let p = Project::new();
    p.write("docs/guide.md", "[missing](absent.md)\n");
    let en = p.run(&["lint", "--format", "json"]);
    let ja = p.run(&["lint", "--format", "json", "--lang", "ja"]);
    assert_eq!(en.status.code(), Some(1));
    assert_eq!(ja.status.code(), Some(1));
    let en: Value = serde_json::from_slice(&en.stdout).unwrap();
    let ja: Value = serde_json::from_slice(&ja.stdout).unwrap();
    assert_eq!(en["status"], ja["status"]);
    assert_eq!(en["summary"], ja["summary"]);
    for key in ["code", "kind", "path", "line", "column", "target"] {
        assert_eq!(en["diagnostics"][0][key], ja["diagnostics"][0][key]);
    }
    assert_eq!(en["diagnostics"][0]["reason"], "Target does not exist");
    assert_ne!(
        en["diagnostics"][0]["reason"],
        ja["diagnostics"][0]["reason"]
    );
    assert_eq!(
        ja["diagnostics"][0]["guidance"]["command"],
        "sdev help documents --lang ja"
    );
}
