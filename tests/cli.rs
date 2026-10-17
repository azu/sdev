use std::{
    collections::BTreeSet,
    env, fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[path = "../src/catalog.rs"]
mod catalog;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = env::temp_dir().join(format!(
            "sdev-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn invoke(args: &[&str]) -> Output {
    let directory = Directory::new();
    let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
    command
        .args(args)
        .current_dir(&directory.0)
        .env_clear()
        .env("PATH", "");
    for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    let result = command.output().unwrap();
    assert_eq!(
        fs::read_dir(&directory.0).unwrap().count(),
        0,
        "CLI must not create files"
    );
    result
}
fn success(args: &[&str]) -> String {
    let result = invoke(args);
    assert!(
        result.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    String::from_utf8(result.stdout).unwrap()
}

#[test]
fn help_selects_guides_by_work_without_exposing_a_file_api() {
    for args in [&[][..], &["--help"][..], &["-h"][..], &["help"][..]] {
        let help = success(args);
        for expected in [
            "sdev help product",
            "sdev help requirements",
            "sdev help design",
            "sdev template",
            "sdev --version",
            "利用先のProduct",
        ] {
            assert!(help.contains(expected));
        }
        for removed in ["docs get", "entry", "licenses", "docs/guidelines/"] {
            assert!(!help.contains(removed));
        }
    }
}

#[test]
fn version_matches_the_package() {
    for arg in ["--version", "-V"] {
        assert_eq!(
            success(&[arg]),
            format!("sdev {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn each_guide_keeps_its_role_and_routes_to_related_commands() {
    let product = success(&["help", "product"]);
    for required in [
        "対象はプロダクト全体",
        "sdev help documents",
        "sdev help workflow",
        "sdev template product",
    ] {
        assert!(product.contains(required), "{required}");
    }
    assert!(!product.contains("](documents.md)"));
    assert!(!product.contains("## 機能要件"));
    let requirements = success(&["help", "requirements"]);
    for required in [
        "親Productを特定して読み",
        "sdev help product",
        "sdev help scenarios",
        "sdev template requirements",
    ] {
        assert!(requirements.contains(required), "{required}");
    }
    let design = success(&["help", "design"]);
    assert!(design.contains("親Productと対象Requirements"));
    assert!(design.contains("sdev template design"));
    for topic in catalog::TOPICS
        .iter()
        .filter(|topic| topic.operation == "help")
    {
        let guide = success(&["help", topic.name]);
        assert!(guide.starts_with("# "));
        assert!(!guide.contains("sdev docs"));
    }
}

#[test]
fn template_overview_explains_the_fallback_and_source_bodies_are_exact() {
    let overview = success(&["template"]);
    assert!(overview.contains("内蔵の既定テンプレート"));
    assert!(overview.contains("利用先の指示・規約"));
    assert!(overview.contains("あれば、そちらを優先"));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for topic in catalog::TOPICS
        .iter()
        .filter(|topic| topic.operation == "template")
    {
        assert!(overview.contains(&format!("sdev template {}", topic.name)));
        assert!(overview.contains(topic.title));
        let original = fs::read_to_string(root.join(topic.source)).unwrap();
        assert_eq!(success(&["template", topic.name]), original);
    }
}

#[test]
fn selection_guide_distinguishes_designated_templates_from_existing_specs() {
    let guide = success(&["help", "documents"]);
    for expected in [
        "対象文書へ適用されるテンプレートがあればそれを優先",
        "任意のMarkdownがあるだけでテンプレートとは扱わず",
        "既存の仕様を変更するときは、その文書の構造を尊重",
        "生成・上書きは行いません",
    ] {
        assert!(guide.contains(expected), "{expected}");
    }
    assert!(success(&["help", "workflow"]).contains("sdev help documents"));
}

#[test]
fn all_referenced_help_and_template_commands_are_reachable() {
    let mut pending = vec![
        ("help".to_string(), "".to_string()),
        ("template".to_string(), "".to_string()),
    ];
    let mut visited = BTreeSet::new();
    while let Some((operation, topic)) = pending.pop() {
        if !visited.insert((operation.clone(), topic.clone())) {
            continue;
        }
        let body = if topic.is_empty() {
            success(&[&operation])
        } else {
            success(&[&operation, &topic])
        };
        for next_operation in ["help", "template"] {
            let prefix = format!("sdev {next_operation} ");
            for suffix in body.split(&prefix).skip(1) {
                let name: String = suffix
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                    .collect();
                if !name.is_empty() {
                    pending.push((next_operation.to_owned(), name));
                }
            }
        }
    }
    let mut expected: BTreeSet<_> = catalog::TOPICS
        .iter()
        .map(|topic| (topic.operation.to_owned(), topic.name.to_owned()))
        .collect();
    expected.extend([
        ("help".to_owned(), String::new()),
        ("template".to_owned(), String::new()),
    ]);
    assert_eq!(visited, expected);
}

#[test]
fn unknown_topics_and_templates_explain_how_to_find_choices() {
    for (operation, name) in [
        ("help", "nonexistent"),
        ("help", "entry"),
        ("template", ""),
        ("template", "../../outside.md"),
    ] {
        let result = invoke(&[operation, name]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains(&format!("sdev {operation}")));
    }
}

#[test]
fn obsolete_and_experiment_operations_and_extra_arguments_are_rejected() {
    for args in [
        &["docs", "list"][..],
        &["docs", "get", "entry"][..],
        &["licenses"][..],
        &["lint"][..],
        &["check", "document.md"][..],
        &["profile", "document.md"][..],
        &["--version", "extra"][..],
        &["help", "product", "extra"][..],
        &["template", "product", "extra"][..],
    ] {
        let result = invoke(args);
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("--help"));
    }
}
