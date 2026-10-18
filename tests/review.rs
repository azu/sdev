use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "sdev-review-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(path.join("hooks")).unwrap();
        let p = Self(path);
        p.git(&["init", "--quiet"]);
        p.git(&["config", "core.excludesFile", ""]);
        p.write(
            "docs/feature/requirements.md",
            "# Behavior\n\nKeep user intent.\n",
        );
        p.write("docs/feature/design.md", "# Method\n\nUse a local file.\n");
        p.write("source.txt", "before\n");
        p.write(".gitignore", "ignored.txt\nhooks/\n");
        p.commit();
        p
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-c")
            .arg(format!("core.hooksPath={}", self.0.join("hooks").display()))
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "user.name=sdev test",
                "-c",
                "user.email=sdev@example.invalid",
            ])
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    fn commit(&self) {
        self.git(&["add", "."]);
        self.git(&["commit", "--quiet", "-m", "test change"]);
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sdev"))
            .arg("review")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
    fn snapshot(&self, args: &[&str]) -> Value {
        let mut arguments = vec!["--prepare"];
        arguments.extend(args);
        let output = self.run(&arguments);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn input(&self, value: &Value) -> PathBuf {
        let path = self.0.join("input.json");
        fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
        path
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn file<'a>(snapshot: &'a Value, path: &str) -> &'a Value {
    snapshot["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == path)
        .unwrap()
}
fn item() -> Value {
    json!({ "id": "R1", "title": "Check intent", "attention": true, "question": "Is the behavior right?",
        "refs": [{"path":"docs/feature/requirements.md", "line":3, "side":"new"}, {"path":"source.txt", "line":1, "side":"new"}],
        "decisions": [{"text":"Local file", "basis":"unknown", "reason":"No recorded reason found in the inspected source."}],
        "examples": [{"label":"An example", "kind":"illustrative", "input":"x", "result":"y"}]
    })
}

#[test]
fn working_tree_captures_staged_unstaged_untracked_and_spec_context() {
    let p = Project::new();
    p.write("source.txt", "staged\n");
    p.git(&["add", "source.txt"]);
    p.write("source.txt", "unstaged 日本語\n");
    p.write("new file.txt", "new\n");
    p.write("ignored.txt", "private\n");
    p.write(
        "docs/feature/requirements.md",
        "# Behavior\n\nChanged intent.\n",
    );
    let snapshot = p.snapshot(&[]);
    assert_eq!(file(&snapshot, "source.txt")["old"], "before\n");
    assert_eq!(file(&snapshot, "source.txt")["new"], "unstaged 日本語\n");
    assert_eq!(file(&snapshot, "new file.txt")["old"], Value::Null);
    assert_eq!(file(&snapshot, "docs/feature/design.md")["changed"], false);
    assert_eq!(snapshot["files"].as_array().unwrap().len(), 4);
    assert_eq!(snapshot["items"], json!([]));
    assert_eq!(snapshot["target"], "working-tree");
}

#[test]
fn explicit_commits_exclude_local_changes_and_preserve_additions_and_deletions() {
    let p = Project::new();
    let base = p.git(&["rev-parse", "HEAD"]);
    fs::remove_file(p.0.join("source.txt")).unwrap();
    p.write("added.txt", "committed\n");
    p.commit();
    let head = p.git(&["rev-parse", "HEAD"]);
    p.write("added.txt", "local\n");
    p.write("untracked.txt", "local\n");
    let snapshot = p.snapshot(&["--base", &base, "--head", &head]);
    assert_eq!(file(&snapshot, "added.txt")["new"], "committed\n");
    assert_eq!(file(&snapshot, "source.txt")["new"], Value::Null);
    assert_eq!(snapshot["files"].as_array().unwrap().len(), 2);
    assert_eq!(snapshot["target"], head);
}

#[test]
fn explicit_spec_is_context_and_empty_comparisons_are_renderable() {
    let p = Project::new();
    assert!(p.snapshot(&[])["files"].as_array().unwrap().is_empty());
    let output = p.run(&["--output", "-"]);
    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("review-data"));
    let snapshot = p.snapshot(&["--spec", "docs/feature/requirements.md"]);
    assert_eq!(snapshot["files"].as_array().unwrap().len(), 2);
    assert!(
        !file(&snapshot, "docs/feature/requirements.md")["changed"]
            .as_bool()
            .unwrap()
    );
}

#[test]
fn binary_large_and_non_utf8_content_are_identified() {
    let p = Project::new();
    fs::write(p.0.join("binary.dat"), b"a\0b").unwrap();
    fs::write(p.0.join("non-utf8.dat"), [0xff]).unwrap();
    fs::write(p.0.join("large.txt"), vec![b'a'; 1024 * 1024 + 1]).unwrap();
    let snapshot = p.snapshot(&[]);
    for path in ["binary.dat", "non-utf8.dat", "large.txt"] {
        assert!(file(&snapshot, path)["notice"].as_str().is_some());
        assert_eq!(file(&snapshot, path)["new"], Value::Null);
    }
}

#[cfg(unix)]
#[test]
fn symlinks_and_symlink_ancestors_do_not_expose_target_text() {
    use std::os::unix::fs::symlink;
    let p = Project::new();
    let outside = Project::new();
    outside.write("secret.txt", "do not include this content\n");
    symlink(outside.0.join("secret.txt"), p.0.join("link.txt")).unwrap();
    fs::remove_dir_all(p.0.join("docs/feature")).unwrap();
    symlink(&outside.0, p.0.join("docs/feature")).unwrap();
    let snapshot = p.snapshot(&[]);
    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(!serialized.contains("do not include this content"));
    assert!(
        file(&snapshot, "link.txt")["notice"]
            .as_str()
            .unwrap()
            .contains("Symbolic link")
    );
    assert!(
        file(&snapshot, "docs/feature/requirements.md")["notice"]
            .as_str()
            .unwrap()
            .contains("Symbolic link")
    );
}

#[test]
fn enriched_snapshot_renders_without_reading_newer_working_tree_text() {
    let p = Project::new();
    p.write("source.txt", "implementation-only evidence\n");
    p.write(
        "docs/feature/requirements.md",
        "# Intent\n\ncaptured value\n",
    );
    let mut snapshot = p.snapshot(&["--spec", "docs/feature/requirements.md"]);
    snapshot["items"] = json!([item()]);
    let path = p.input(&snapshot);
    p.write("docs/feature/requirements.md", "# Intent\n\nnewer value\n");
    let output = p.run(&[
        "--input",
        path.to_str().unwrap(),
        "--output",
        "-",
        "--lang",
        "ja",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("captured value"));
    assert!(!html.contains("implementation-only evidence"));
    let embedded = html
        .split("type=\"application/json\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let presentation: Value = serde_json::from_str(embedded).unwrap();
    assert!(
        presentation["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["role"] != "implementation")
    );
    assert_eq!(
        presentation["items"][0]["refs"].as_array().unwrap().len(),
        1
    );
    assert!(!html.contains("newer value"));
    assert!(html.contains("lang=\"ja\""));
    assert!(html.contains("THIRD") || html.contains("Third-party notices"));
    assert!(html.contains("connect-src 'none'"));
}

#[test]
fn hostile_text_is_data_and_raw_markdown_html_is_inert() {
    let p = Project::new();
    p.write(
        "docs/feature/requirements.md",
        "# Title\n\n<script>attack()</script>\n\n![pixel](https://example.invalid/x)\n",
    );
    let mut snapshot = p.snapshot(&[]);
    snapshot["title"] = "</title><script>attack()</script>".into();
    let path = p.input(&snapshot);
    let output = p.run(&["--input", path.to_str().unwrap(), "--output", "-"]);
    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(!html.contains("<script>attack()</script>"));
    assert!(html.contains("\\u003cscript"));
    assert!(!html.contains("<img src=\"https://example.invalid"));
}

#[test]
fn invalid_references_enums_and_unknown_fields_fail_before_writing() {
    let p = Project::new();
    p.write("source.txt", "after\n");
    let mut snapshot = p.snapshot(&["--spec", "docs/feature/requirements.md"]);
    snapshot["items"] = json!([item()]);
    for (field, value) in [
        ("line", json!(999)),
        ("line", json!(0)),
        ("path", json!("absent")),
        ("side", json!("after")),
    ] {
        let mut invalid = snapshot.clone();
        invalid["items"][0]["refs"][0][field] = value;
        let path = p.input(&invalid);
        let output = p.run(&["--input", path.to_str().unwrap()]);
        assert_eq!(output.status.code(), Some(1));
        assert!(!p.0.join("sdev-review.html").exists());
    }
    for (collection, field) in [("decisions", "basis"), ("examples", "kind")] {
        let mut invalid = snapshot.clone();
        invalid["items"][0][collection][0][field] = "wrong".into();
        let path = p.input(&invalid);
        assert_eq!(
            p.run(&["--input", path.to_str().unwrap()]).status.code(),
            Some(1)
        );
    }
    snapshot["items"][0]["typo"] = true.into();
    let path = p.input(&snapshot);
    assert_eq!(
        p.run(&["--input", path.to_str().unwrap()]).status.code(),
        Some(1)
    );
}

#[test]
fn relationship_endpoints_and_reasons_are_validated_without_inventing_links() {
    let p = Project::new();
    p.write("source.txt", "after\n");
    let mut snapshot = p.snapshot(&["--spec", "docs/feature/requirements.md"]);
    let mut linked = item();
    linked["refs"][0]["id"] = "R1".into();
    linked["refs"][0]["label"] = "Keep intent".into();
    linked["refs"][1]["id"] = "C1".into();
    linked["links"] =
        json!([{"from":"R1","to":"C1","reason":"This branch implements the required behavior."}]);
    snapshot["items"] = json!([linked]);
    let input = p.input(&snapshot);
    assert!(
        p.run(&["--input", input.to_str().unwrap(), "--output", "-"])
            .status
            .success()
    );
    for (field, value) in [("from", "absent"), ("to", "R1"), ("reason", " ")] {
        let mut invalid = snapshot.clone();
        invalid["items"][0]["links"][0][field] = value.into();
        let input = p.input(&invalid);
        assert_eq!(
            p.run(&["--input", input.to_str().unwrap()]).status.code(),
            Some(1)
        );
        assert!(!p.0.join("sdev-review.html").exists());
    }
    let mut duplicate = snapshot.clone();
    duplicate["items"][0]["refs"][1]["id"] = "R1".into();
    let input = p.input(&duplicate);
    assert_eq!(
        p.run(&["--input", input.to_str().unwrap()]).status.code(),
        Some(1)
    );
    let link = snapshot["items"][0]["links"][0].clone();
    snapshot["items"][0]["links"]
        .as_array_mut()
        .unwrap()
        .push(link);
    let input = p.input(&snapshot);
    assert_eq!(
        p.run(&["--input", input.to_str().unwrap()]).status.code(),
        Some(1)
    );
}

#[test]
fn existing_output_is_preserved_and_invalid_options_are_rejected() {
    let p = Project::new();
    p.write("source.txt", "after\n");
    p.write("existing.html", "keep me");
    assert_eq!(p.run(&["--output", "existing.html"]).status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(p.0.join("existing.html")).unwrap(),
        "keep me"
    );
    for args in [
        vec!["--unknown"],
        vec!["--base"],
        vec!["--base", "HEAD", "--base", "HEAD"],
        vec!["--prepare", "--prepare"],
        vec!["--serve", "--prepare"],
        vec!["--serve", "--output", "x"],
        vec!["--port", "0"],
        vec!["--serve", "--port", "65536"],
        vec!["--serve", "--port", "x"],
        vec!["--input", "x", "--prepare"],
        vec!["--input", "x", "--base", "HEAD"],
        vec!["--input", "x", "--spec", "requirements.md"],
    ] {
        assert_eq!(p.run(&args).status.code(), Some(2), "{args:?}");
    }
    assert_eq!(
        p.run(&["--base", "--not-a-revision"]).status.code(),
        Some(1)
    );
    assert_eq!(
        p.run(&["--spec", "../requirements.md"]).status.code(),
        Some(1)
    );
}

#[test]
fn input_rendering_does_not_require_git_or_the_source_repository() {
    let p = Project::new();
    p.write("source.txt", "after\n");
    let input = p.input(&p.snapshot(&[]));
    let output = Command::new(env!("CARGO_BIN_EXE_sdev"))
        .args([
            "review",
            "--input",
            input.to_str().unwrap(),
            "--output",
            "-",
        ])
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn missing_specifications_accept_an_assessment_without_fabricated_references() {
    let p = Project::new();
    let mut snapshot = p.snapshot(&[]);
    snapshot["items"] = json!([{
        "id":"MISSING", "title":"A new user-visible rule", "attention":true, "refs":[],
        "question":"Should this rule be recorded?",
        "assessment":{"kind":"missing_requirement", "summary":"A rule was found", "impact":"Users see it", "inspected":"Inspected the changed branch; intent remains unknown"}
    }]);
    let input = p.input(&snapshot);
    let result = p.run(&["--input", input.to_str().unwrap(), "--output", "-"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("A rule was found")
    );
    for (field, value) in [("kind", "incorrect"), ("impact", " "), ("inspected", "")] {
        let mut invalid = snapshot.clone();
        invalid["items"][0]["assessment"][field] = value.into();
        let input = p.input(&invalid);
        assert_eq!(
            p.run(&["--input", input.to_str().unwrap(), "--output", "-"])
                .status
                .code(),
            Some(1)
        );
    }
}

struct ReviewServer {
    child: std::process::Child,
    url: String,
    stdout: std::io::BufReader<std::process::ChildStdout>,
}
impl ReviewServer {
    fn start(project: &Project, args: &[&str]) -> Self {
        use std::io::BufRead;
        let mut child = Command::new(env!("CARGO_BIN_EXE_sdev"))
            .args(["review", "--serve"])
            .args(args)
            .env("JEV_API_KEY", "test-key-never-sent")
            .current_dir(&project.0)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut line = String::new();
        let mut stdout = std::io::BufReader::new(child.stdout.take().unwrap());
        stdout.read_line(&mut line).unwrap();
        assert!(line.starts_with("http://127.0.0.1:"), "{line}");
        Self {
            child,
            url: line.trim().to_owned(),
            stdout,
        }
    }
    fn data(&self) -> Value {
        let html = ureq::get(&self.url)
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();
        serde_json::from_str(
            html.split("<script id=\"review-data\" type=\"application/json\">")
                .nth(1)
                .unwrap()
                .split("</script>")
                .next()
                .unwrap(),
        )
        .unwrap()
    }
    fn refresh(&self, revision: &str) -> Value {
        ureq::post(format!("{}api/refresh", self.url))
            .header("Content-Type", "application/json")
            .header("Origin", self.url.trim_end_matches('/'))
            .send_json(json!({"revision":revision}))
            .unwrap()
            .body_mut()
            .read_json()
            .unwrap()
    }
}
impl Drop for ReviewServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn working_tree_server_hot_updates_actual_specifications_and_code_and_rejects_stale_judgments() {
    let p = Project::new();
    p.write(
        "docs/feature/requirements.md",
        "# Behavior\n\nThe initial behavior.\n",
    );
    let cache = fs::canonicalize(p.0.join("hooks")).unwrap().join("cache");
    let server = ReviewServer::start(&p, &["--jev", "--cache-dir", cache.to_str().unwrap()]);
    let before = server.data();
    let revision = before["revision"].as_str().unwrap();
    assert_eq!(before["hot"], true);
    assert!(server.refresh(revision)["data"].is_null());
    std::thread::sleep(std::time::Duration::from_millis(2300));
    assert!(
        server.refresh(revision)["data"].is_null(),
        "Capture timestamps alone must not trigger updates"
    );
    p.write(
        "docs/feature/requirements.md",
        "# Behavior\n\nThe changed behavior.\n",
    );
    p.write("docs/feature/design.md", "# Method\n\nA changed method.\n");
    p.write("source.txt", "changed implementation\n");
    p.write("new-code.txt", "new implementation\n");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let after = loop {
        let result = server.refresh(revision);
        if !result["data"].is_null() {
            break result["data"].clone();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "The working tree was not refreshed"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    assert_ne!(after["revision"], before["revision"]);
    assert_eq!(after["base"], before["base"]);
    assert!(
        file(&after, "docs/feature/requirements.md")["new"]
            .as_str()
            .unwrap()
            .contains("changed behavior")
    );
    assert!(
        file(&after, "docs/feature/design.md")["new_html"]
            .as_str()
            .unwrap()
            .contains("changed method")
    );
    assert!(
        after["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["role"] != "implementation")
    );
    assert!(
        after["code_scope"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == "new-code.txt")
    );
    assert_eq!(server.data()["revision"], after["revision"]);
    let stale=ureq::post(format!("{}api/related",server.url))
        .header("Content-Type", "application/json")
        .header("Origin",server.url.trim_end_matches('/'))
        .send_json(json!({"revision":revision,"path":"docs/feature/requirements.md","line":3,"side":"new"}));
    assert!(matches!(stale, Err(ureq::Error::StatusCode(409))));
    let cross_origin = ureq::post(format!("{}api/refresh", server.url))
        .header("Content-Type", "application/json")
        .header("Origin", "https://attacker.invalid")
        .send_json(json!({"revision":revision}));
    assert!(matches!(cross_origin, Err(ureq::Error::StatusCode(403))));
    fs::remove_file(p.0.join("docs/feature/design.md")).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let result = server.refresh(after["revision"].as_str().unwrap());
        if !result["data"].is_null() {
            assert!(file(&result["data"], "docs/feature/design.md")["new"].is_null());
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let stable = server.data();
    let stable_revision = stable["revision"].as_str().unwrap();
    fs::rename(p.0.join(".git"), p.0.join("hooks/hidden-git")).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let result = server.refresh(stable_revision);
        if !result["error"].is_null() {
            assert!(result["data"].is_null());
            assert_eq!(result["revision"], stable_revision);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    fs::rename(p.0.join("hooks/hidden-git"), p.0.join(".git")).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let result = server.refresh(stable_revision);
        if result["error"].is_null() {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
#[test]
fn prepared_and_two_commit_servers_remain_fixed() {
    let p = Project::new();
    let input = p.input(&p.snapshot(&["--spec", "docs/feature/requirements.md"]));
    for args in [
        vec!["--input", input.to_str().unwrap()],
        vec!["--head", "HEAD", "--spec", "docs/feature/requirements.md"],
    ] {
        let server = ReviewServer::start(&p, &args);
        assert_eq!(server.data()["hot"], false);
        let refresh: Value = ureq::post(format!("{}api/refresh", server.url))
            .header("Content-Type", "application/json")
            .header("Origin", server.url.trim_end_matches('/'))
            .send_json(json!({"revision":"old"}))
            .unwrap()
            .body_mut()
            .read_json()
            .unwrap();
        assert!(refresh["data"].is_null());
        assert_eq!(refresh["revision"], server.data()["revision"]);
    }
}

#[test]
fn server_feedback_delivers_once_to_the_launch_output_and_never_to_jev() {
    use std::io::BufRead;
    let p = Project::new();
    let input = p.input(&p.snapshot(&["--spec", "docs/feature/requirements.md"]));
    let mut server = ReviewServer::start(&p, &["--input", input.to_str().unwrap()]);
    let data = server.data();
    assert_eq!(data["server"], true);
    let snapshot = json!({"id":data["id"],"base":data["base"],"target":data["target"],"captured_at":data["captured_at"],"revision":data["revision"]});
    let submission = json!({"id":"feedback-example","report":{"version":1,"snapshot":snapshot,"notes":[{"id":"comment-example","text":"Explain the export method.","reference":{"path":"docs/feature/requirements.md","line":3,"side":"new","quote":"Keep user intent."},"item":null}]}});
    let send = |body: &Value, origin: &str| {
        ureq::post(format!("{}api/feedback", server.url))
            .header("Content-Type", "application/json")
            .header("Origin", origin)
            .send_json(body)
    };
    let origin = server.url.trim_end_matches('/');
    assert!(matches!(
        send(&submission, "https://attacker.invalid"),
        Err(ureq::Error::StatusCode(403))
    ));
    let receipt: Value = send(&submission, origin)
        .unwrap()
        .body_mut()
        .read_json()
        .unwrap();
    assert_eq!(receipt["delivered"], true);
    assert_eq!(receipt["duplicate"], false);
    let duplicate: Value = send(&submission, origin)
        .unwrap()
        .body_mut()
        .read_json()
        .unwrap();
    assert_eq!(duplicate["duplicate"], true);
    let mut invalid = submission.clone();
    invalid["report"]["notes"][0]["text"] = "Different text".into();
    assert!(matches!(
        send(&invalid, origin),
        Err(ureq::Error::StatusCode(400))
    ));
    invalid["id"] = "other-feedback".into();
    invalid["report"]["notes"][0]["reference"]["path"] = "../outside.md".into();
    assert!(matches!(
        send(&invalid, origin),
        Err(ureq::Error::StatusCode(400))
    ));
    let mut line = String::new();
    server.stdout.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "Ctrl-C to stop.");
    line.clear();
    server.stdout.read_line(&mut line).unwrap();
    let delivered: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(delivered["type"], "review.feedback");
    assert_eq!(delivered["report"], submission["report"]);
    let resolve = |id: &str, revision: &str, summary: &str, origin: &str| {
        ureq::post(format!("{}api/feedback/resolve", server.url))
            .header("Origin", origin)
            .header("Content-Type", "application/json")
            .send_json(json!({"id":id,"revision":revision,"summary":summary}))
    };
    let revision = data["revision"].as_str().unwrap();
    for (id, revision, summary) in [
        ("unknown", revision, "Fixed"),
        ("comment-example", "stale", "Fixed"),
        ("comment-example", revision, ""),
    ] {
        assert!(matches!(
            resolve(id, revision, summary, origin),
            Err(ureq::Error::StatusCode(400))
        ));
    }
    assert!(matches!(
        resolve(
            "comment-example",
            revision,
            "Fixed",
            "https://attacker.invalid"
        ),
        Err(ureq::Error::StatusCode(403))
    ));
    assert!(
        server.refresh(revision)["resolutions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let resolved: Value = resolve(
        "comment-example",
        revision,
        "Verified the Clipboard API explanation.",
        origin,
    )
    .unwrap()
    .body_mut()
    .read_json()
    .unwrap();
    assert_eq!(resolved["resolved"], true);
    let status = server.refresh(revision);
    assert!(status["data"].is_null());
    assert_eq!(status["resolutions"][0]["id"], "comment-example");
    assert_eq!(status["resolutions"][0]["snapshot"]["revision"], revision);
    assert_eq!(
        status["resolutions"][0]["summary"],
        "Verified the Clipboard API explanation."
    );
    let mut conflict = submission.clone();
    conflict["id"] = "second-submission".into();
    conflict["report"]["notes"][0]["text"] = "Different comment".into();
    assert!(matches!(
        send(&conflict, origin),
        Err(ureq::Error::StatusCode(400))
    ));
}
