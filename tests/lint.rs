use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sdev-lint-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("docs")).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, body: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sdev"));
        command
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", "")
            .args(args);
        for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        command.output().unwrap()
    }
    fn report(&self, code: i32) -> Value {
        let result = self.run(&["lint", "--format", "json"]);
        assert_eq!(
            result.status.code(),
            Some(code),
            "{} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert!(result.stdout.ends_with(b"\n"));
        report
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
const PRODUCT: &str = "docs/products/a/product.md";
const REQUIREMENTS: &str = "docs/products/a/features/f/requirements.md";
const DESIGN: &str = "docs/products/a/features/f/design.md";
fn codes(report: &Value) -> Vec<&str> {
    report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect()
}

#[test]
fn defaults_and_json_check_markdown_without_writes_or_runtime_path() {
    let p = Project::new();
    p.write("README.md", "[入口](docs/guide.md#日本語)\n");
    p.write("AGENTS.md", "[同じ入口](/docs/guide.md)\n");
    p.write(
        "docs/guide.md",
        "# 日本語\n[外部](https://example.invalid/not-requested)\n",
    );
    p.write("elsewhere.md", "[対象外](missing.md)\n");
    let before = fs::read(p.0.join("docs/guide.md")).unwrap();
    let report = p.report(0);
    assert_eq!(report["status"], "pass");
    assert_eq!(report["summary"]["documents"], 3);
    assert_eq!(report["summary"]["internal_references"], 2);
    assert_eq!(report["summary"]["external_references"], 1);
    let default = p.run(&["lint"]);
    let explicit = p.run(&["lint", "--format", "text"]);
    assert!(default.status.success());
    assert!(default.stderr.is_empty());
    assert_eq!(default.stdout, explicit.stdout);
    assert!(
        String::from_utf8(default.stdout)
            .unwrap()
            .contains("pass: 検査完了、違反なし")
    );
    assert_eq!(fs::read(p.0.join("docs/guide.md")).unwrap(), before);
    assert_eq!(fs::read_dir(&p.0).unwrap().count(), 4);
}

#[test]
fn full_direction_matrix_including_cross_product_and_same_document() {
    for source in [PRODUCT, REQUIREMENTS, DESIGN] {
        let p = Project::new();
        for path in [
            PRODUCT,
            REQUIREMENTS,
            DESIGN,
            "docs/products/b/product.md",
            "docs/products/b/features/g/requirements.md",
            "docs/products/b/features/g/design.md",
            "docs/engineering/design/common.md",
            "docs/guidelines/rule.md",
            "src/lib.rs",
        ] {
            p.write(path, "# Target\n");
        }
        p.write(source, "# Here\n[self](#here)\n[p](/docs/products/b/product.md)\n[r](/docs/products/b/features/g/requirements.md)\n[d](/docs/products/b/features/g/design.md)\n[common](/docs/engineering/design/common.md)\n[g](/docs/guidelines/rule.md)\n[c](/src/lib.rs)\n");
        let report = p.report(1);
        assert_eq!(codes(&report), vec!["forbidden-reference"; 3], "{source}");
        assert_eq!(report["summary"]["failures"], 0);
        for diag in report["diagnostics"].as_array().unwrap() {
            assert_eq!(diag["path"], source);
            assert_eq!(diag["guidance"]["command"], "sdev help documents");
            assert_eq!(diag["guidance"]["section"], "文書の参照方向");
        }
    }
}

#[test]
fn guidelines_templates_and_general_documents_only_check_links() {
    let p = Project::new();
    p.write(DESIGN, "# Target\n");
    p.write("src/lib.rs", "");
    for source in [
        "docs/guidelines/example.md",
        "docs/templates/design.md",
        "docs/notes.MD",
    ] {
        p.write(
            source,
            "[d](/docs/products/a/features/f/design.md)\n[c](/src/lib.rs)\n",
        );
    }
    let report = p.report(0);
    assert_eq!(report["summary"]["direction_documents"], 1);
    assert_eq!(report["summary"]["link_only_documents"], 3);
    p.write("src/comment.rs", "// [missing](missing.md)\n");
    p.report(0);
    p.write("docs/guidelines/example.md", "[missing](missing.md)\n");
    assert_eq!(codes(&p.report(1)), ["missing-target"]);
}

#[test]
fn used_reference_definitions_images_and_unicode_positions_ignore_code_html_metadata() {
    let p = Project::new();
    p.write("docs/a.md", "---\nvalue: '[metadata](missing-meta.md)'\n---\n```markdown\n[fenced](missing-fence.md)\n```\n\n    [indented](missing-indent.md)\n\n`[inline](missing-inline.md)`\n<!-- [comment](missing-comment.md) -->\n<a href=\"missing-html.md\">html</a>\n\n[unused]: missing-unused.md\n[undefined]\n\n日本語\t[リンク][ref]\n![画像][image]\n\n[ref]: missing.md\n[image]: missing.png\n");
    let report = p.report(1);
    assert_eq!(codes(&report), ["missing-target", "missing-target"]);
    let d = &report["diagnostics"];
    assert_eq!(d[0]["line"], 17);
    assert_eq!(d[0]["column"], 5);
    assert_eq!(d[0]["target"], "missing.md");
    assert_eq!(d[1]["line"], 18);
    assert_eq!(d[1]["column"], 1);
    assert_eq!(d[1]["target"], "missing.png");
}

#[test]
fn anchors_include_japanese_decorations_collisions_setext_images_and_percent_encoding() {
    let p = Project::new();
    p.write("docs/target.md", "# A\n# A\n# A-1\n# A\n# 日本語 **強調** `code` ![画像](ok.png) <b>HTML</b> &amp;\nSetext\n===\n# 😄 emoji\n");
    p.write("docs/ok.png", "");
    p.write("docs/source.md", "[a](target.md#a) [a1](target.md#a-1) [a11](target.md#a-1-1) [a2](target.md#a-2)\n[日本語](target.md#%E6%97%A5%E6%9C%AC%E8%AA%9E-強調-code-画像-html-)\n[s](target.md#setext) [e](target.md#-emoji) [top](target.md#)\n");
    p.report(0);
    p.write("docs/source.md", "[not slugged again](target.md#Setext)\n");
    assert_eq!(codes(&p.report(1)), ["missing-heading"]);
}

#[test]
fn footnotes_gfm_table_and_alert_links_are_checked_at_use_locations() {
    let p = Project::new();
    p.write("docs/gfm.md", "| column |\n| --- |\n| [table](missing-table.md) |\n\n> [!NOTE]\n> [alert](missing-alert.md)\n\n- [x] [task](missing-task.md)\n\ntext[^one]\n\n[^one]: [footnote](missing-footnote.md)\n");
    let report = p.report(1);
    assert_eq!(report["summary"]["internal_references"], 4);
    assert_eq!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [3, 6, 8, 12]
    );
}

#[test]
fn url_resolution_decodes_once_preserves_plus_and_skips_non_markdown_fragments() {
    let p = Project::new();
    p.write("docs/space + %.md", "# A+B\n");
    p.write("docs/a%20b.md", "# X\n");
    p.write("docs/a.png", "");
    p.write("docs/sub/source.md", "[a](../space%20+%20%25.md?ignored=yes#ab)\n[b](/docs/a%2520b.md#x)\n[image](../a.png#any)\n[directory](../sub/)\n[dot](./../sub/../a.png)\n[external](//example.invalid/a) [file](file:///outside) [mail](mailto:x@y.invalid) [data](data:text/plain,x)\n");
    let report = p.report(0);
    assert_eq!(report["summary"]["internal_references"], 5);
    assert_eq!(report["summary"]["external_references"], 4);
    assert_eq!(report["summary"]["skipped_heading_checks"], 1);
}

#[test]
fn missing_and_forbidden_are_independent_and_diagnostics_have_stable_order() {
    let p = Project::new();
    p.write(
        REQUIREMENTS,
        "[d](design.md#missing)\n[c](/src/missing.rs)\n[p](../../product.md)\n[u](/no.something)\n",
    );
    let report = p.report(1);
    assert_eq!(
        codes(&report),
        [
            "forbidden-reference",
            "missing-target",
            "forbidden-reference",
            "missing-target",
            "missing-target",
            "missing-target"
        ]
    );
    assert_eq!(
        p.run(&["lint", "--format", "json"]).stdout,
        p.run(&["lint", "--format", "json"]).stdout
    );
}

#[test]
fn incomplete_keeps_violations_and_reports_unknown_existing_types_and_invalid_utf8() {
    let p = Project::new();
    p.write(
        PRODUCT,
        "[unknown](/unknown.blob)\n[missing](/missing.md)\n",
    );
    p.write("unknown.blob", "");
    p.write("docs/bad.md", [0xff, 0xfe]);
    let report = p.report(2);
    assert_eq!(report["status"], "incomplete");
    assert_eq!(report["summary"]["documents"], 1);
    assert_eq!(report["summary"]["violations"], 1);
    assert_eq!(report["summary"]["failures"], 2);
    assert_eq!(
        codes(&report),
        ["invalid-utf8", "unclassified-target", "missing-target"]
    );
    let failure = &report["diagnostics"][0];
    assert!(
        failure["line"].is_null() && failure["column"].is_null() && failure["target"].is_null()
    );
    let text = p.run(&["lint"]);
    assert_eq!(text.status.code(), Some(2));
    assert!(text.stderr.is_empty());
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("docs/bad.md [invalid-utf8]"));
    assert!(text.contains("incomplete: 検査未完了"));
}

#[test]
fn absence_of_docs_or_all_targets_is_incomplete_and_arguments_are_separate() {
    let p = Project::new();
    p.report(2);
    fs::remove_dir(p.0.join("docs")).unwrap();
    p.write("README.md", "# Readme\n");
    assert_eq!(codes(&p.report(2)), ["discovery-failed"]);
    for args in [
        vec!["lint", "--format", "xml"],
        vec!["lint", "--format"],
        vec!["lint", "docs"],
        vec!["lint", "--root", "."],
        vec!["lint", "--fix"],
        vec!["lint", "--format", "json", "extra"],
    ] {
        let result = p.run(&args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("sdev lint"));
    }
    for args in [&["lint", "--help"][..], &["lint", "-h"]] {
        let result = p.run(args);
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        assert!(String::from_utf8_lossy(&result.stdout).contains("docs/**/*.md"));
    }
}

#[test]
fn invalid_encodings_windows_paths_and_root_escape_are_differentiated() {
    let p = Project::new();
    p.write("docs/a.md", "[percent](bad%xx.md)\n[utf8](%FF.md)\n[nul](bad%00.md)\n[separator](bad%2Fname.md)\n[backslash](bad%5Cname.md)\n[windows](C:/outside.md)\n[unc](<\\\\server\\share\\outside.md>)\n[outside](../../outside.md)\n");
    let report = p.report(2);
    assert_eq!(report["summary"]["violations"], 5);
    assert!(codes(&report).contains(&"unsupported-path"));
    assert!(codes(&report).contains(&"outside-root"));
}

#[test]
fn out_of_scope_target_headings_do_not_expand_discovery() {
    let p = Project::new();
    p.write("docs/guide.md", "[target](/notes.md#there)\n");
    p.write("notes.md", "# There\n[not followed](missing.md)\n");
    let report = p.report(0);
    assert_eq!(report["summary"]["documents"], 1);
    assert_eq!(report["summary"]["internal_references"], 1);
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_escape_and_same_design_alias_keeps_identity() {
    use std::os::unix::fs::symlink;
    let p = Project::new();
    let outside = Project::new();
    outside.write("secret.md", "# Secret\n");
    symlink(&outside.0, p.0.join("escape")).unwrap();
    p.write(DESIGN, "# Here\n[self](/alias.md#here)\n[escape](/escape/secret.md#secret)\n[missing outside](/escape/absent.md)\n");
    symlink(p.0.join(DESIGN), p.0.join("alias.md")).unwrap();
    symlink(&outside.0, p.0.join("docs/unsafe")).unwrap();
    let report = p.report(2);
    assert_eq!(
        codes(&report),
        ["outside-root", "outside-root", "discovery-failed"]
    );
    assert!(!codes(&report).contains(&"forbidden-reference"));
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_names_are_failures() {
    use std::os::unix::ffi::OsStrExt;
    let p = Project::new();
    p.write("docs/ok.md", "# OK\n");
    let name = std::ffi::OsStr::from_bytes(b"bad-\xff.md");
    fs::write(p.0.join("docs").join(name), "").unwrap();
    let report = p.report(2);
    assert_eq!(codes(&report), ["unsupported-path"]);
    assert!(
        report["diagnostics"][0]["path"]
            .as_str()
            .unwrap()
            .contains("\\xff")
    );
}

#[test]
fn own_documentation_passes_with_fixed_placement_and_no_exceptions() {
    let result = Command::new(env!("CARGO_BIN_EXE_sdev"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["lint", "--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["diagnostics"], serde_json::json!([]), "{report:#}");
    assert!(result.status.success());
}

#[test]
fn crlf_shortcut_and_collapsed_references_keep_unicode_columns() {
    let p = Project::new();
    p.write("docs/crlf.md", "# 見出し\r\n[ok](#見出し)\r\n日本語\t[missing][] [short]\r\n\r\n[missing]: absent.md\r\n[short]: absent-too.md\r\n");
    let report = p.report(1);
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0]["line"], 3);
    assert_eq!(diagnostics[0]["column"], 5);
    assert_eq!(diagnostics[1]["line"], 3);
    assert_eq!(diagnostics[1]["column"], 17);
}

#[cfg(unix)]
#[test]
fn unreadable_documents_are_failures_while_readable_violations_remain() {
    use std::os::unix::fs::PermissionsExt;
    let p = Project::new();
    p.write("docs/private.md", "# Private\n");
    p.write("docs/readable.md", "[missing](absent.md)\n");
    let path = p.0.join("docs/private.md");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    // Root may retain access; the mode-denial scenario needs a non-root process.
    let denied = fs::read(&path).is_err();
    if denied {
        let report = p.report(2);
        assert_eq!(codes(&report), ["read-failed", "missing-target"]);
        assert_eq!(report["summary"]["documents"], 1);
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
fn entry_names_must_be_regular_files_and_do_not_expand_discovery() {
    let p = Project::new();
    p.write("README.md/child.md", "[missing](absent.md)\n");
    p.write("docs/ok.md", "# OK\n");
    let report = p.report(2);
    assert_eq!(codes(&report), ["discovery-failed"]);
    assert_eq!(report["summary"]["documents"], 1);
}

#[test]
fn text_diagnostics_escape_control_characters_and_json_retains_them() {
    let p = Project::new();
    p.write("docs/control.md", "# Here\n[missing](#bad&#x1B;name)\n");
    let report = p.report(1);
    assert_eq!(report["diagnostics"][0]["target"], "#bad\u{1b}name");
    let result = p.run(&["lint"]);
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("\\u{1b}"));
}
