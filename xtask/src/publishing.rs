use crate::{
    release,
    support::{self, Result, TemporaryDirectory, Toolchain, root},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn stable_version(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.bytes().all(|c| c.is_ascii_digit())
                && (p.len() == 1 || !p.starts_with('0'))
                && p.parse::<u64>().is_ok()
        })
}
fn validate_version(version: &str, actual: &str, license: &[u8]) -> Result<()> {
    if !stable_version(version) {
        return Err("Specify a stable X.Y.Z version without a v prefix".into());
    }
    if version != actual {
        return Err(format!(
            "Requested {version}, Cargo.toml contains {actual}; update and merge the version first"
        )
        .into());
    }
    if license.iter().all(u8::is_ascii_whitespace) {
        return Err("A nonempty project LICENSE is required before publishing".into());
    }
    Ok(())
}
fn check_version(version: &str) -> Result<()> {
    let metadata: Value =
        serde_json::from_str(&support::output(Toolchain::new()?.cargo().args([
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ]))?)?;
    let package = metadata["packages"]
        .as_array()
        .ok_or("Missing packages")?
        .iter()
        .find(|p| p["name"] == "sdev")
        .ok_or("Missing sdev")?;
    let license = fs::read(root().join("LICENSE")).map_err(
        |_| "Choose the project license and commit a nonempty root LICENSE before publishing",
    )?;
    validate_version(
        version,
        package["version"]
            .as_str()
            .ok_or("Missing package version")?,
        &license,
    )
}
struct Context {
    repository: String,
    version: String,
    sha: String,
    run: String,
}
impl Context {
    fn environment(version: &str) -> Result<Self> {
        check_version(version)?;
        if env::var("GITHUB_EVENT_NAME")? != "workflow_dispatch"
            || env::var("GITHUB_REF")? != "refs/heads/main"
        {
            return Err("Release only from workflow_dispatch on main".into());
        }
        let repository = env::var("GITHUB_REPOSITORY")?;
        let parts: Vec<_> = repository.split('/').collect();
        if parts.len() != 2
            || parts.iter().any(|p| {
                p.is_empty()
                    || !p
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
            })
        {
            return Err("Invalid repository".into());
        }
        let sha = env::var("GITHUB_SHA")?;
        let run = env::var("GITHUB_RUN_ID")?;
        if sha.len() != 40
            || !sha.bytes().all(|c| c.is_ascii_hexdigit())
            || run.is_empty()
            || !run.bytes().all(|c| c.is_ascii_digit())
            || release::source_commit()? != sha
        {
            return Err("Invalid or mismatched release source/run".into());
        }
        Ok(Self {
            repository,
            version: version.into(),
            sha,
            run,
        })
    }
    fn tag(&self) -> String {
        format!("v{}", self.version)
    }
    fn marker(&self) -> String {
        format!("<!-- sdev-release source={} run={} -->", self.sha, self.run)
    }
    fn endpoint(&self, path: &str) -> String {
        format!("repos/{}/{path}", self.repository)
    }
}

trait Api {
    fn request(&mut self, method: &str, path: &str, body: Option<Value>) -> Result<Option<Value>>;
    fn upload(&mut self, context: &Context, files: &Path) -> Result<()>;
}
struct Gh;
fn response(output: std::process::Output) -> Result<Option<Value>> {
    let text = String::from_utf8(output.stdout)?;
    let (headers, body) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .ok_or_else(|| {
            format!(
                "Missing API response: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let code: u16 = headers
        .lines()
        .next()
        .and_then(|s| s.split_whitespace().nth(1))
        .ok_or("Missing HTTP status")?
        .parse()?;
    if code == 404 {
        return Ok(None);
    }
    if !output.status.success() || !(200..300).contains(&code) {
        return Err(format!("GitHub API failed ({code}): {body}").into());
    }
    if body.trim().is_empty() {
        Ok(Some(Value::Null))
    } else {
        Ok(Some(serde_json::from_str(body)?))
    }
}
impl Api for Gh {
    fn request(&mut self, method: &str, path: &str, body: Option<Value>) -> Result<Option<Value>> {
        let mut command = Command::new("gh");
        command.args([
            "api",
            "--hostname",
            "github.com",
            "--include",
            "--method",
            method,
            "-H",
            "Accept: application/vnd.github+json",
            path,
        ]);
        if body.is_some() {
            command.args(["--input", "-"]).stdin(Stdio::piped());
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(body) = body {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(serde_json::to_string(&body)?.as_bytes())?;
        }
        response(child.wait_with_output()?)
    }
    fn upload(&mut self, context: &Context, files: &Path) -> Result<()> {
        let mut paths: Vec<_> = fs::read_dir(files)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<_>>()?;
        paths.sort();
        support::run(
            Command::new("gh")
                .args([
                    "release",
                    "upload",
                    &context.tag(),
                    "--repo",
                    &context.repository,
                    "--clobber",
                ])
                .args(paths),
        )
    }
}
fn required(api: &mut impl Api, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
    api.request(method, path, body)?
        .ok_or_else(|| format!("Required GitHub resource not found: {path}").into())
}
fn list(api: &mut impl Api, path: &str) -> Result<Vec<Value>> {
    let mut all = Vec::new();
    for page in 1.. {
        let result = required(
            api,
            "GET",
            &format!("{path}?per_page=100&page={page}"),
            None,
        )?;
        let entries = result.as_array().ok_or("Expected API list")?;
        all.extend(entries.iter().cloned());
        if entries.len() < 100 {
            return Ok(all);
        }
    }
    unreachable!()
}
fn state(api: &mut impl Api, context: &Context) -> Result<(bool, Option<Value>)> {
    let main = required(api, "GET", &context.endpoint("git/ref/heads/main"), None)?;
    if main["object"]["sha"] != context.sha {
        return Err(
            "main moved after dispatch; inspect any draft and dispatch the current main instead"
                .into(),
        );
    }
    let tag = api.request(
        "GET",
        &context.endpoint(&format!("git/ref/tags/{}", context.tag())),
        None,
    )?;
    if let Some(tag) = &tag
        && (tag["object"]["type"] != "commit" || tag["object"]["sha"] != context.sha)
    {
        return Err("Existing tag points elsewhere or is not this workflow's lightweight tag; never move it".into());
    }
    let releases = list(api, &context.endpoint("releases"))?;
    let existing: Vec<_> = releases
        .into_iter()
        .filter(|r| r["tag_name"] == context.tag())
        .collect();
    if existing.len() > 1 {
        return Err("Multiple releases for one version".into());
    }
    let release = existing.into_iter().next();
    if let Some(release) = &release {
        if release["draft"] != true {
            return Err("Release already published; inspect it, do not overwrite".into());
        }
        if release["target_commitish"] != context.sha
            || !release["body"]
                .as_str()
                .unwrap_or("")
                .lines()
                .any(|line| line == context.marker())
        {
            return Err("Existing draft belongs to another source or run; rerun the original run or inspect it manually".into());
        }
    }
    Ok((tag.is_some(), release))
}
pub fn preflight(version: &str) -> Result<()> {
    let context = Context::environment(version)?;
    state(&mut Gh, &context)?;
    println!(
        "Release source {} / {} validated; no tag or release changed",
        context.sha,
        context.tag()
    );
    Ok(())
}

fn regular(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!("Expected regular artifact: {}", path.display()).into());
    }
    Ok(fs::read(path)?)
}
fn bundle(
    context: &Context,
    artifacts: &Path,
    out: &Path,
    source: &Path,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    let expected: BTreeSet<_> = release::TARGETS
        .iter()
        .map(|t| {
            format!(
                "release-{}-{}",
                context.run,
                release::asset_name(&context.version, t)
            )
        })
        .collect();
    let found: BTreeSet<_> = fs::read_dir(artifacts)?
        .map(|entry| {
            Ok(entry?
                .file_name()
                .into_string()
                .map_err(|_| "Invalid artifact name")?)
        })
        .collect::<Result<_>>()?;
    if found != expected {
        return Err("Missing or unexpected target artifacts".into());
    }
    for target in release::TARGETS {
        let asset = release::asset_name(&context.version, target);
        let directory = artifacts.join(format!("release-{}-{asset}", context.run));
        if fs::symlink_metadata(&directory)?.file_type().is_symlink() {
            return Err("Artifact directory cannot be a symlink".into());
        }
        let binary = directory.join(&asset);
        let metadata: Value =
            serde_json::from_slice(&regular(&support::sidecar(&binary, ".json"))?)?;
        let bytes = regular(&binary)?;
        let checksum = regular(&support::sidecar(&binary, ".sha256"))?;
        let digest = support::digest(&bytes);
        if metadata["source_commit"] != context.sha
            || metadata["version"] != context.version
            || metadata["target"] != target.name
            || metadata["verified"] != true
            || metadata["source_clean"] != true
            || metadata["published"] != false
            || metadata["sha256"] != digest
            || metadata["bytes"].as_u64() != Some(bytes.len() as u64)
            || metadata["binary"] != format!("dist/{asset}")
            || checksum != format!("{digest}  {asset}\n").as_bytes()
        {
            return Err(format!("Unverified, mismatched or damaged artifact: {asset}").into());
        }
        if metadata["notices"] != "THIRD-PARTY-NOTICES.html"
            || metadata["crate_notices"] != "THIRD-PARTY-CRATES.html"
        {
            return Err("Unexpected notices".into());
        }
        files.insert(asset.clone(), bytes);
        files.insert(format!("{asset}.sha256"), checksum);
        for name in [
            "LICENSE",
            "THIRD-PARTY-NOTICES.html",
            "THIRD-PARTY-CRATES.html",
        ] {
            let bytes = regular(&directory.join(name))?;
            if let Some(previous) = files.get(name)
                && previous != &bytes
            {
                return Err(format!("Shared notice/license differs across targets: {name}").into());
            }
            if name == "LICENSE" && bytes != regular(&source.join("LICENSE"))? {
                return Err("Artifact LICENSE differs from release source".into());
            }
            if bytes.is_empty() {
                return Err(format!("Empty notice/license: {name}").into());
            }
            files.insert(name.into(), bytes);
        }
    }
    fs::create_dir_all(out)?;
    if fs::read_dir(out)?.next().is_some() {
        return Err("Release bundle output must be empty".into());
    }
    for (name, bytes) in &files {
        fs::write(out.join(name), bytes)?;
    }
    Ok(files)
}
fn publish(
    api: &mut impl Api,
    context: &Context,
    files: &Path,
    expected: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let (tag, draft) = state(api, context)?;
    if !tag {
        required(
            api,
            "POST",
            &context.endpoint("git/refs"),
            Some(json!({"ref":format!("refs/tags/{}", context.tag()), "sha":context.sha})),
        )?;
    }
    let draft = match draft {
        Some(draft) => draft,
        None => required(
            api,
            "POST",
            &context.endpoint("releases"),
            Some(
                json!({"tag_name":context.tag(),"target_commitish":context.sha,"name":context.tag(),"draft":true,"prerelease":false,"body":format!("{}\n\nSource commit: {}\n\n単体実行ファイル・SHA-256・ライセンスと依存通知を添付しています。",context.marker(),context.sha)}),
            ),
        )?,
    };
    let id = draft["id"].as_u64().ok_or("Missing release id")?;
    api.upload(context, files)?;
    let uploaded = list(api, &context.endpoint(&format!("releases/{id}/assets")))?;
    if uploaded.len() != expected.len() {
        return Err("Uploaded asset set differs; leave draft for inspection".into());
    }
    let names: BTreeSet<_> = uploaded.iter().filter_map(|a| a["name"].as_str()).collect();
    if names != expected.keys().map(String::as_str).collect() {
        return Err("Uploaded names differ from the complete bundle".into());
    }
    for asset in uploaded {
        let bytes = expected
            .get(asset["name"].as_str().ok_or("Missing asset name")?)
            .ok_or("Unexpected release asset")?;
        if asset["state"] != "uploaded"
            || asset["size"].as_u64() != Some(bytes.len() as u64)
            || asset["digest"] != format!("sha256:{}", support::digest(bytes))
        {
            return Err("Uploaded asset digest/state mismatch; keep draft".into());
        }
    }
    // Recheck after upload: a moved tag or externally published draft must stop us.
    let (tag, release) = state(api, context)?;
    if !tag || release.as_ref().and_then(|r| r["id"].as_u64()) != Some(id) {
        return Err("Release state changed while uploading".into());
    }
    let published = required(
        api,
        "PATCH",
        &context.endpoint(&format!("releases/{id}")),
        Some(json!({"draft":false,"make_latest":"legacy"})),
    )?;
    if published["draft"] != false || published["tag_name"] != context.tag() {
        return Err(
            "Publication response was not confirmed; inspect the release before retrying".into(),
        );
    }
    let confirmed = required(
        api,
        "GET",
        &context.endpoint(&format!("releases/{id}")),
        None,
    )?;
    if confirmed["draft"] != false || confirmed["tag_name"] != context.tag() {
        return Err("Published state was not confirmed".into());
    }
    println!("Published {} from {}", context.tag(), context.sha);
    Ok(())
}
pub fn run(version: &str, artifacts: &Path) -> Result<()> {
    let context = Context::environment(version)?;
    let output = TemporaryDirectory::new("release-bundle")?;
    let files = bundle(&context, artifacts, &output.0, &root())?;
    publish(&mut Gh, &context, &output.0, &files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn context() -> Context {
        Context {
            repository: "owner/repo".into(),
            version: "0.1.0".into(),
            sha: "a".repeat(40),
            run: "123".into(),
        }
    }
    fn draft(c: &Context) -> Value {
        json!({"id":1,"tag_name":c.tag(),"target_commitish":c.sha,"draft":true,"body":c.marker()})
    }
    struct Fake {
        context: Context,
        tag: Option<Value>,
        release: Option<Value>,
        events: Vec<String>,
        assets: Vec<Value>,
        expected: BTreeMap<String, Vec<u8>>,
        upload_fails: bool,
        corrupt: bool,
        moved: bool,
    }
    impl Fake {
        fn new() -> Self {
            Self {
                context: context(),
                tag: None,
                release: None,
                events: Vec::new(),
                assets: Vec::new(),
                expected: BTreeMap::from([
                    ("binary".into(), b"content".to_vec()),
                    ("notice".into(), b"license".to_vec()),
                ]),
                upload_fails: false,
                corrupt: false,
                moved: false,
            }
        }
    }
    impl Api for Fake {
        fn request(
            &mut self,
            method: &str,
            path: &str,
            body: Option<Value>,
        ) -> Result<Option<Value>> {
            self.events.push(format!("{method} {path}"));
            let value = if method == "GET" && path.ends_with("git/ref/heads/main") {
                Some(
                    json!({"object":{"sha":if self.moved { "b".repeat(40) } else { self.context.sha.clone() }}}),
                )
            } else if method == "GET" && path.contains("git/ref/tags/") {
                self.tag.clone()
            } else if method == "GET" && path.contains("releases?per_page") {
                Some(json!(self.release.iter().collect::<Vec<_>>()))
            } else if method == "POST" && path.ends_with("git/refs") {
                assert_eq!(body.unwrap()["sha"], self.context.sha);
                self.tag = Some(json!({"object":{"type":"commit","sha":self.context.sha}}));
                self.tag.clone()
            } else if method == "POST" && path.ends_with("releases") {
                assert_eq!(body.as_ref().unwrap()["draft"], true);
                assert_eq!(body.unwrap()["target_commitish"], self.context.sha);
                self.release = Some(draft(&self.context));
                self.release.clone()
            } else if method == "GET" && path.contains("/assets?") {
                Some(json!(self.assets))
            } else if method == "PATCH" {
                assert_eq!(body.unwrap()["draft"], false);
                self.release.as_mut().unwrap()["draft"] = json!(false);
                self.release.clone()
            } else if method == "GET" && path.ends_with("releases/1") {
                self.release.clone()
            } else {
                panic!("Unexpected {method} {path}");
            };
            Ok(value)
        }
        fn upload(&mut self, _: &Context, _: &Path) -> Result<()> {
            self.events.push("UPLOAD".into());
            if self.upload_fails {
                return Err("network interrupted".into());
            }
            self.assets = self.expected.iter().map(|(name, bytes)| json!({"name":name,"state":"uploaded","size":bytes.len(),"digest":format!("sha256:{}",support::digest(bytes))})).collect();
            if self.corrupt {
                self.assets[0]["digest"] = json!("sha256:wrong");
            }
            Ok(())
        }
    }
    #[test]
    fn github_responses_distinguish_absence_from_denial_or_network_failure() {
        fn output(code: i32, text: &str) -> std::process::Output {
            #[cfg(unix)]
            let status = {
                use std::os::unix::process::ExitStatusExt;
                std::process::ExitStatus::from_raw(code << 8)
            };
            #[cfg(windows)]
            let status = {
                use std::os::windows::process::ExitStatusExt;
                std::process::ExitStatus::from_raw(code as u32)
            };
            std::process::Output {
                status,
                stdout: text.as_bytes().to_vec(),
                stderr: b"API error".to_vec(),
            }
        }
        assert_eq!(
            response(output(
                0,
                "HTTP/1.1 200 OK\r\nX-Test: value\r\n\r\n{\"id\":1}"
            ))
            .unwrap()
            .unwrap()["id"],
            1
        );
        assert!(
            response(output(
                1,
                "HTTP/2.0 404 Not Found\n\n{\"message\":\"Not Found\"}"
            ))
            .unwrap()
            .is_none()
        );
        assert!(
            response(output(
                1,
                "HTTP/2.0 403 Forbidden\n\n{\"message\":\"Denied\"}"
            ))
            .is_err()
        );
        assert!(response(output(1, "")).is_err());
        assert_eq!(
            response(output(0, "HTTP/2.0 204 No Content\n\n")).unwrap(),
            Some(Value::Null)
        );
    }
    #[test]
    fn version_and_license_gate_rejects_unsafe_or_mismatched_input() {
        validate_version("0.1.0", "0.1.0", b"MIT license").unwrap();
        for version in [
            "",
            "v0.1.0",
            "0.1",
            "01.1.0",
            "0.1.0-rc.1",
            "0.1.0+build",
            "0.1.0;echo unsafe",
            "../0.1.0",
        ] {
            assert!(
                validate_version(version, version, b"license").is_err(),
                "{version}"
            );
        }
        assert!(validate_version("0.2.0", "0.1.0", b"license").is_err());
        assert!(validate_version("0.1.0", "0.1.0", b" \n\t").is_err());
    }
    #[test]
    fn fresh_publish_creates_tag_then_draft_and_publishes_only_after_asset_verification() {
        let mut api = Fake::new();
        let c = context();
        let expected = api.expected.clone();
        publish(&mut api, &c, Path::new("unused"), &expected).unwrap();
        let tag = api
            .events
            .iter()
            .position(|e| e.ends_with("git/refs"))
            .unwrap();
        let draft = api
            .events
            .iter()
            .position(|e| e == "POST repos/owner/repo/releases")
            .unwrap();
        let upload = api.events.iter().position(|e| e == "UPLOAD").unwrap();
        let verify = api
            .events
            .iter()
            .position(|e| e.contains("/assets?"))
            .unwrap();
        let published = api
            .events
            .iter()
            .position(|e| e.starts_with("PATCH "))
            .unwrap();
        assert!(tag < draft && draft < upload && upload < verify && verify < published);
        assert_eq!(
            api.events.last().unwrap(),
            "GET repos/owner/repo/releases/1"
        );
    }
    #[test]
    fn existing_published_foreign_draft_wrong_tag_or_moved_main_never_mutate() {
        for case in 0..5 {
            let mut api = Fake::new();
            match case {
                0 => {
                    let mut r = draft(&api.context);
                    r["draft"] = json!(false);
                    api.release = Some(r);
                }
                1 => {
                    let mut r = draft(&api.context);
                    r["body"] = json!("other run");
                    api.release = Some(r);
                }
                2 => api.tag = Some(json!({"object":{"type":"commit","sha":"b".repeat(40)}})),
                3 => api.tag = Some(json!({"object":{"type":"tag","sha":api.context.sha}})),
                _ => api.moved = true,
            }
            let expected = api.expected.clone();
            assert!(publish(&mut api, &context(), Path::new("unused"), &expected).is_err());
            assert!(api.events.iter().all(|event| event.starts_with("GET ")));
        }
    }
    #[test]
    fn upload_failure_or_corrupt_digest_keeps_draft_and_same_run_can_resume() {
        for fail_upload in [true, false] {
            let mut api = Fake::new();
            api.upload_fails = fail_upload;
            api.corrupt = !fail_upload;
            let expected = api.expected.clone();
            assert!(publish(&mut api, &context(), Path::new("unused"), &expected).is_err());
            assert_eq!(api.release.as_ref().unwrap()["draft"], true);
            assert!(!api.events.iter().any(|e| e.starts_with("PATCH ")));
            api.events.clear();
            api.upload_fails = false;
            api.corrupt = false;
            publish(&mut api, &context(), Path::new("unused"), &expected).unwrap();
            assert!(!api.events.iter().any(|e| e.starts_with("POST ")));
        }
    }
    fn artifacts(c: &Context, base: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let input = base.join("artifacts");
        let source = base.join("source");
        let out = base.join("out");
        fs::create_dir(&input).unwrap();
        fs::create_dir(&source).unwrap();
        fs::write(source.join("LICENSE"), "chosen license").unwrap();
        for target in release::TARGETS {
            let name = release::asset_name(&c.version, target);
            let directory = input.join(format!("release-{}-{name}", c.run));
            fs::create_dir(&directory).unwrap();
            let binary = directory.join(&name);
            let bytes = b"sample artifact for hash validation";
            let digest = support::digest(bytes);
            fs::write(&binary, bytes).unwrap();
            fs::write(
                support::sidecar(&binary, ".sha256"),
                format!("{digest}  {name}\n"),
            )
            .unwrap();
            fs::write(support::sidecar(&binary, ".json"), json!({"source_commit":c.sha,"version":c.version,"target":target.name,"source_clean":true,"verified":true,"published":false,"sha256":digest,"bytes":bytes.len(),"binary":format!("dist/{name}"),"notices":"THIRD-PARTY-NOTICES.html","crate_notices":"THIRD-PARTY-CRATES.html"}).to_string()).unwrap();
            for (name, body) in [
                ("LICENSE", "chosen license"),
                ("THIRD-PARTY-NOTICES.html", "rust notice"),
                ("THIRD-PARTY-CRATES.html", "crate notice"),
            ] {
                fs::write(directory.join(name), body).unwrap();
            }
        }
        (input, source, out)
    }
    #[test]
    fn bundle_requires_all_verified_targets_and_copies_only_public_assets() {
        let tmp = TemporaryDirectory::new("bundle-test").unwrap();
        let c = context();
        let (input, source, out) = artifacts(&c, &tmp.0);
        let result = bundle(&c, &input, &out, &source).unwrap();
        assert_eq!(result.len(), release::TARGETS.len() * 2 + 3);
        assert!(result.keys().all(|name| !name.ends_with(".json")));
        for (name, bytes) in result {
            assert_eq!(fs::read(out.join(name)).unwrap(), bytes);
        }
    }
    #[test]
    fn damaged_incomplete_mismatched_or_unverified_artifacts_never_stage_a_bundle() {
        for case in 0..8 {
            let tmp = TemporaryDirectory::new("invalid-bundle").unwrap();
            let c = context();
            let (input, source, out) = artifacts(&c, &tmp.0);
            let name = release::asset_name(&c.version, &release::TARGETS[0]);
            let dir = input.join(format!("release-{}-{name}", c.run));
            let binary = dir.join(&name);
            let meta = support::sidecar(&binary, ".json");
            match case {
                0 => fs::write(&binary, "damaged").unwrap(),
                1 => fs::remove_dir_all(&dir).unwrap(),
                2..=5 => {
                    let mut m: Value = serde_json::from_slice(&fs::read(&meta).unwrap()).unwrap();
                    match case {
                        2 => m["verified"] = json!(false),
                        3 => m["source_commit"] = json!("b".repeat(40)),
                        4 => m["version"] = json!("0.2.0"),
                        _ => m["target"] = json!("other"),
                    };
                    fs::write(&meta, m.to_string()).unwrap();
                }
                6 => fs::write(dir.join("LICENSE"), "other license").unwrap(),
                _ => fs::write(dir.join("THIRD-PARTY-CRATES.html"), "other notices").unwrap(),
            }
            assert!(bundle(&c, &input, &out, &source).is_err(), "case {case}");
            assert!(!out.exists());
        }
    }
}
