use crate::language::Language;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

mod feedback;
mod live;
mod server;
mod table;

const FILE_LIMIT: usize = 1024 * 1024;
const TOTAL_LIMIT: usize = 16 * FILE_LIMIT;
const INPUT_LIMIT: usize = 48 * FILE_LIMIT;
const VIEWER: &str = include_str!("viewer.bundle.js");
const STYLE: &str = include_str!("viewer.css");
const NOTICES: &str = include_str!("THIRD-PARTY-NOTICES.txt");

#[derive(Clone, Default)]
pub struct Arguments {
    base: Option<String>,
    head: Option<String>,
    specs: Vec<String>,
    input: Option<String>,
    output: Option<String>,
    prepare: bool,
    serve: bool,
    port: Option<u16>,
    jev: bool,
    jev_model: Option<String>,
    cache_dir: Option<String>,
}

pub fn arguments(args: &[&str]) -> Result<Arguments, String> {
    let mut result = Arguments::default();
    let mut args = args.iter();
    while let Some(&flag) = args.next() {
        if flag == "--prepare" && !result.prepare {
            result.prepare = true;
            continue;
        }
        if flag == "--serve" && !result.serve {
            result.serve = true;
            continue;
        }
        if flag == "--jev" && !result.jev {
            result.jev = true;
            continue;
        }
        let value = args
            .next()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("Invalid review arguments: {flag}. See sdev review --help."))?;
        let slot = match flag {
            "--port" => {
                let port = value
                    .parse::<u16>()
                    .map_err(|_| "--port must be 0–65535.")?;
                if result.port.replace(port).is_some() {
                    return Err("Specify --port only once.".into());
                }
                continue;
            }
            "--base" => &mut result.base,
            "--jev-model" => &mut result.jev_model,
            "--cache-dir" => &mut result.cache_dir,
            "--head" => &mut result.head,
            "--input" => &mut result.input,
            "--output" => &mut result.output,
            "--spec" => {
                result.specs.push((*value).to_owned());
                continue;
            }
            _ => {
                return Err(format!(
                    "Invalid review argument: {flag}. See sdev review --help."
                ));
            }
        };
        if slot.replace((*value).to_owned()).is_some() {
            return Err(format!("Specify {flag} only once."));
        }
    }
    if result.input.is_some()
        && (result.base.is_some()
            || result.head.is_some()
            || !result.specs.is_empty()
            || result.prepare)
    {
        return Err("--input cannot be combined with --base, --head, --spec, or --prepare.".into());
    }
    if result.serve && (result.prepare || result.output.is_some()) {
        return Err("--serve cannot be combined with --prepare or --output.".into());
    }
    if result.port.is_some() && !result.serve {
        return Err("--port requires --serve.".into());
    }
    if (result.jev && !result.serve)
        || (!result.jev && (result.jev_model.is_some() || result.cache_dir.is_some()))
    {
        return Err("--jev requires --serve; --jev-model and --cache-dir require --jev.".into());
    }
    Ok(result)
}

pub fn help(language: Language) -> &'static str {
    language.select(include_str!("help.txt"), include_str!("help.ja.txt"))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u8,
    id: String,
    title: String,
    captured_at: u64,
    base: String,
    target: String,
    files: Vec<ReviewFile>,
    items: Vec<Item>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewFile {
    path: String,
    role: String,
    changed: bool,
    old: Option<String>,
    new: Option<String>,
    notice: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    path: String,
    line: usize,
    side: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    id: String,
    title: String,
    attention: bool,
    #[serde(default)]
    question: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    assessment: Option<Assessment>,
    refs: Vec<Reference>,
    #[serde(default)]
    links: Vec<Relationship>,
    #[serde(default)]
    decisions: Vec<Decision>,
    #[serde(default)]
    examples: Vec<Example>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Assessment {
    kind: String,
    summary: String,
    impact: String,
    inspected: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Relationship {
    from: String,
    to: String,
    reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    text: String,
    basis: String,
    reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Example {
    label: String,
    kind: String,
    input: String,
    result: String,
}

fn path_ok(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', '\0', ':'])
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(value) if value != ".git"))
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let result = Command::new("git")
        .args(["--literal-pathspecs", "-c", "core.fsmonitor=false", "-C"])
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("Cannot run Git: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "Git failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    Ok(result.stdout)
}

fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(git(root, args)?).map_err(|e| format!("Git output is not UTF-8: {e}"))
}

fn paths(root: &Path, args: &[&str]) -> Result<BTreeSet<String>, String> {
    git(root, args)?
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(|e| format!("Path is not UTF-8: {e}")))
        .collect()
}

fn revision(root: &Path, value: &str) -> Result<String, String> {
    let result = git_text(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{value}^{{commit}}"),
        ],
    )?;
    let result = result.trim();
    if !matches!(result.len(), 40 | 64) || !result.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Git returned an invalid commit ID.".into());
    }
    Ok(result.to_owned())
}

fn role(path: &str) -> &'static str {
    let name = Path::new(path)
        .file_name()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    for role in ["requirements", "design"] {
        if name == format!("{role}.md")
            || (name.starts_with(&format!("{role}.")) && name.ends_with(".md"))
        {
            return role;
        }
    }
    "implementation"
}

fn decode(bytes: Vec<u8>) -> Result<Option<String>, String> {
    if bytes.contains(&0) {
        return Err("Binary file; text is unavailable.".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "Non-UTF-8 file; text is unavailable.".into())
}

fn tree_text(root: &Path, commit: &str, path: &str) -> Result<Option<String>, String> {
    let entry = git_text(root, &["ls-tree", "-z", commit, "--", path])?;
    if entry.is_empty() {
        return Ok(None);
    }
    let header = entry.split_once('\t').ok_or("Invalid Git tree entry.")?.0;
    let mut words = header.split_whitespace();
    let mode = words.next().ok_or("Missing Git file mode.")?;
    let kind = words.next().ok_or("Missing Git object type.")?;
    let object = words.next().ok_or("Missing Git object ID.")?;
    if !matches!(mode, "100644" | "100755") || kind != "blob" {
        return Err("Symbolic link, directory, or submodule; text is unavailable.".into());
    }
    let size: usize = git_text(root, &["cat-file", "-s", object])?
        .trim()
        .parse()
        .map_err(|_| "Invalid Git blob size.")?;
    if size > FILE_LIMIT {
        return Err("File exceeds the 1 MiB text limit.".into());
    }
    decode(git(root, &["cat-file", "blob", object])?)
}

fn local_text(root: &Path, path: &str) -> Result<Option<String>, String> {
    let mut location = root.to_path_buf();
    for part in Path::new(path).components() {
        location.push(part);
        match fs::symlink_metadata(&location) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("Symbolic link; text is unavailable.".into());
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Cannot inspect file: {e}")),
        }
    }
    let metadata = fs::metadata(&location).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("Directory or submodule; text is unavailable.".into());
    }
    if metadata.len() > FILE_LIMIT as u64 {
        return Err("File exceeds the 1 MiB text limit.".into());
    }
    let bytes = read_limited(&location, FILE_LIMIT)?;
    decode(bytes)
}

fn read_limited(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds the input size limit.", path.display()));
    }
    Ok(bytes)
}

fn capture(root: &Path, args: &Arguments) -> Result<Snapshot, String> {
    let git_root = git_text(root, &["rev-parse", "--show-toplevel"])?;
    if fs::canonicalize(git_root.trim()).map_err(|e| e.to_string())?
        != fs::canonicalize(root).map_err(|e| e.to_string())?
    {
        return Err("Run sdev review at the Git project root.".into());
    }
    let base = revision(root, args.base.as_deref().unwrap_or("HEAD"))?;
    let head = args
        .head
        .as_deref()
        .map(|r| revision(root, r))
        .transpose()?;
    let mut diff_args = vec![
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--name-only",
        "-z",
        &base,
    ];
    if let Some(head) = &head {
        diff_args.push(head);
    }
    diff_args.push("--");
    let mut changed = paths(root, &diff_args)?;
    let mut available = paths(root, &["ls-tree", "-r", "--name-only", "-z", &base])?;
    if let Some(head) = &head {
        available.extend(paths(root, &["ls-tree", "-r", "--name-only", "-z", head])?);
    } else {
        changed.extend(paths(
            root,
            &["ls-files", "--others", "--exclude-standard", "-z"],
        )?);
        available.extend(paths(
            root,
            &[
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ],
        )?);
    }
    let mut selected = changed.clone();
    for spec in &args.specs {
        if !path_ok(spec) || role(spec) == "implementation" || !available.contains(spec) {
            return Err(format!(
                "--spec must name an existing repository-relative requirements[.LANG].md or design[.LANG].md: {spec}"
            ));
        }
        selected.insert(spec.clone());
    }
    let directories: BTreeSet<_> = selected
        .iter()
        .filter(|p| role(p) != "implementation")
        .filter_map(|p| Path::new(p).parent().map(Path::to_path_buf))
        .collect();
    selected.extend(available.into_iter().filter(|p| {
        role(p) != "implementation"
            && Path::new(p)
                .parent()
                .is_some_and(|d| directories.contains(d))
    }));
    if selected.len() > 500 {
        return Err(
            "The comparison exceeds the 500-file limit; choose a narrower comparison.".into(),
        );
    }
    let mut files = Vec::new();
    let mut total = 0;
    for path in selected {
        if !path_ok(&path) {
            return Err(format!("Unsupported repository-relative path: {path:?}"));
        }
        let mut notices = Vec::new();
        let mut read = |result: Result<Option<String>, String>, side: &str| match result {
            Ok(Some(text)) if total + text.len() <= TOTAL_LIMIT => {
                total += text.len();
                Some(text)
            }
            Ok(Some(_)) => {
                notices.push(format!("{side}: Comparison exceeds the 16 MiB text limit."));
                None
            }
            Ok(None) => None,
            Err(e) => {
                notices.push(format!("{side}: {e}"));
                None
            }
        };
        let old = read(tree_text(root, &base, &path), "old");
        let new = read(
            match &head {
                Some(head) => tree_text(root, head, &path),
                None => local_text(root, &path),
            },
            "new",
        );
        files.push(ReviewFile {
            role: role(&path).into(),
            changed: changed.contains(&path),
            path,
            old,
            new,
            notice: (!notices.is_empty()).then(|| notices.join("\n")),
        });
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(Snapshot {
        version: 1,
        id: format!("review-{}-{}", now.as_nanos(), std::process::id()),
        title: "Specification review".into(),
        captured_at: now.as_secs(),
        base,
        target: head.unwrap_or_else(|| "working-tree".into()),
        files,
        items: Vec::new(),
        warnings: Vec::new(),
    })
}

fn validate(snapshot: &Snapshot) -> Result<(), String> {
    if snapshot.version != 1 || snapshot.id.trim().is_empty() {
        return Err("Unsupported review version or missing snapshot ID.".into());
    }
    if snapshot.files.len() > 500 || snapshot.items.len() > 2000 {
        return Err("Review exceeds the file or item limit.".into());
    }
    let mut files = BTreeMap::new();
    let mut total = 0;
    for file in &snapshot.files {
        if !path_ok(&file.path)
            || !["requirements", "design", "implementation"].contains(&file.role.as_str())
            || files.insert(&file.path, file).is_some()
        {
            return Err(format!("Invalid or duplicate review file: {}", file.path));
        }
        for text in [&file.old, &file.new].into_iter().flatten() {
            if text.len() > FILE_LIMIT || text.contains('\0') {
                return Err(format!("Invalid or oversized text: {}", file.path));
            }
            total += text.len();
        }
    }
    if total > TOTAL_LIMIT {
        return Err("Review exceeds the 16 MiB text limit.".into());
    }
    let mut ids = BTreeSet::new();
    for item in &snapshot.items {
        if item.id.trim().is_empty() || item.title.trim().is_empty() || !ids.insert(&item.id) {
            return Err(format!("Invalid or duplicate review item: {}", item.id));
        }
        if let Some(assessment) = &item.assessment
            && (![
                "aligned",
                "missing_requirement",
                "missing_design",
                "mismatch",
                "uncertain",
            ]
            .contains(&assessment.kind.as_str())
                || [
                    &assessment.summary,
                    &assessment.impact,
                    &assessment.inspected,
                ]
                .iter()
                .any(|value| value.trim().is_empty()))
        {
            return Err(format!("{}: Invalid specification assessment.", item.id));
        }
        let mut reference_ids = BTreeSet::new();
        for reference in &item.refs {
            if let Some(id) = &reference.id
                && (id.trim().is_empty() || !reference_ids.insert(id))
            {
                return Err(format!("{}: Invalid or duplicate reference ID.", item.id));
            }
            let file = files
                .get(&reference.path)
                .ok_or_else(|| format!("{}: Missing reference file {}", item.id, reference.path))?;
            let text = match reference.side.as_str() {
                "old" => &file.old,
                "new" => &file.new,
                _ => return Err(format!("{}: Reference side must be old or new.", item.id)),
            };
            if reference.line == 0
                || text
                    .as_ref()
                    .is_none_or(|t| reference.line > t.lines().count())
            {
                return Err(format!(
                    "{}: Reference outside captured text: {}:{} ({})",
                    item.id, reference.path, reference.line, reference.side
                ));
            }
        }
        let mut pairs = BTreeSet::new();
        for link in &item.links {
            if !reference_ids.contains(&link.from)
                || !reference_ids.contains(&link.to)
                || link.from == link.to
                || link.reason.trim().is_empty()
                || !pairs.insert((&link.from, &link.to))
            {
                return Err(format!(
                    "{}: Invalid relationship endpoints, duplicate link, or missing reason.",
                    item.id
                ));
            }
        }
        for decision in &item.decisions {
            if !["recorded", "inferred", "unknown"].contains(&decision.basis.as_str()) {
                return Err(format!("{}: Invalid decision basis.", item.id));
            }
        }
        for example in &item.examples {
            if !["observed", "illustrative", "unverified"].contains(&example.kind.as_str()) {
                return Err(format!("{}: Invalid example kind.", item.id));
            }
        }
    }
    Ok(())
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn markdown(text: &str) -> String {
    let line_starts: Vec<_> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect();
    let mut table_rows = Vec::new();
    let events = Parser::new_ext(
        text,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .into_offset_iter()
    .map(|(event, range)| {
        let line = line_starts.partition_point(|start| *start <= range.start);
        let end_line = line_starts
            .partition_point(|start| *start < range.end)
            .max(line);
        match event {
            Event::Html(content) | Event::InlineHtml(content) => Event::Text(content),
            Event::Start(Tag::Paragraph) => Event::Html(
                format!("<p tabindex=\"0\" data-line=\"{line}\" data-line-end=\"{end_line}\">")
                    .into(),
            ),
            Event::Start(Tag::Heading { level, .. }) => Event::Html(
                format!(
                    "<{level} tabindex=\"0\" data-line=\"{line}\" data-line-end=\"{end_line}\">"
                )
                .into(),
            ),
            Event::Start(Tag::CodeBlock(_)) => Event::Html(
                format!(
                    "<pre tabindex=\"0\" data-line=\"{line}\" data-line-end=\"{end_line}\"><code>"
                )
                .into(),
            ),
            Event::Start(Tag::Item) => Event::Html(
                format!("<li tabindex=\"0\" data-line=\"{line}\" data-line-end=\"{end_line}\">")
                    .into(),
            ),
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                // Keep the renderer's header/alignment state intact.
                table_rows.push((line, end_line));
                event
            }
            Event::Start(Tag::Link { .. }) => Event::Html("<span class=\"inert-link\">".into()),
            Event::End(TagEnd::Link) => Event::Html("</span>".into()),
            Event::Start(Tag::Image { .. }) => Event::Text("[image: ".into()),
            Event::End(TagEnd::Image) => Event::Text("]".into()),
            event => event,
        }
    });
    let mut result = String::new();
    html::push_html(&mut result, events);
    if table_rows.is_empty() {
        return result;
    }
    let mut located = String::new();
    for (index, part) in result.split("<tr>").enumerate() {
        if index > 0 {
            if let Some((line, end)) = table_rows.get(index - 1) {
                located.push_str(&format!(
                    "<tr tabindex=\"0\" data-line=\"{line}\" data-line-end=\"{end}\">"
                ));
            } else {
                located.push_str("<tr>");
            }
        }
        located.push_str(part);
    }
    located
}

fn presentation(
    snapshot: &Snapshot,
    language: Language,
    live: bool,
) -> Result<serde_json::Value, String> {
    let mut data = serde_json::to_value(snapshot).map_err(|e| e.to_string())?;
    for (value, file) in data["files"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(&snapshot.files)
    {
        if file.role != "implementation" {
            value["old_html"] = file.old.as_deref().map(markdown).into();
            value["new_html"] = file.new.as_deref().map(markdown).into();
        }
    }
    // Implementation is investigation material for the agent, not a third reading pane.
    let spec_paths: BTreeSet<_> = snapshot
        .files
        .iter()
        .filter(|file| file.role != "implementation")
        .map(|file| file.path.as_str())
        .collect();
    data["files"]
        .as_array_mut()
        .unwrap()
        .retain(|file| file["role"].as_str() != Some("implementation"));
    for item in data["items"].as_array_mut().unwrap() {
        item["refs"].as_array_mut().unwrap().retain(|reference| {
            reference["path"]
                .as_str()
                .is_some_and(|path| spec_paths.contains(path))
        });
        let ids: BTreeSet<String> = item["refs"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|reference| reference["id"].as_str().map(str::to_owned))
            .collect();
        item["links"].as_array_mut().unwrap().retain(|link| {
            link["from"].as_str().is_some_and(|id| ids.contains(id))
                && link["to"].as_str().is_some_and(|id| ids.contains(id))
        });
    }
    data["comparison_has_changes"] = snapshot.files.iter().any(|file| file.changed).into();
    data["language"] = language.code().into();
    data["live"] = live.into();
    if live {
        data["code_scope"] = live::scope(snapshot, language);
    }
    Ok(data)
}

fn render(snapshot: &Snapshot, language: Language, live: bool) -> Result<String, String> {
    render_data(&presentation(snapshot, language, live)?, language, live)
}

fn render_data(
    data: &serde_json::Value,
    language: Language,
    connected: bool,
) -> Result<String, String> {
    let title = escape(data["title"].as_str().unwrap_or("Specification review"));
    let data = serde_json::to_string(&data)
        .map_err(|e| e.to_string())?
        .replace('<', "\\u003c");
    let viewer = VIEWER.replace("</script", "<\\/script");
    let page = format!(
        "<!doctype html>\n<html lang=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'\"><title>{} · sdev review</title><style>{STYLE}</style></head><body><div id=\"app\"></div><noscript>Enable JavaScript to browse this review.</noscript><details class=\"licenses\"><summary>Third-party notices</summary><pre>{}</pre></details><script id=\"review-data\" type=\"application/json\">{data}</script><script>{viewer}</script></body></html>\n",
        language.code(),
        title,
        escape(NOTICES)
    );
    Ok(if connected {
        page.replace("connect-src 'none'", "connect-src 'self'")
    } else {
        page
    })
}

pub fn run(
    args: &Arguments,
    root: &Path,
    writer: &mut impl Write,
    language: Language,
) -> Result<(), String> {
    let snapshot: Snapshot = if let Some(input) = &args.input {
        serde_json::from_slice(&read_limited(Path::new(input), INPUT_LIMIT)?)
            .map_err(|e| format!("Invalid review input: {e}"))?
    } else {
        capture(root, args)?
    };
    validate(&snapshot)?;
    let content = if args.prepare {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?
        )
    } else {
        render(&snapshot, language, args.jev)?
    };
    if args.serve {
        let live = if args.jev {
            let directory = args
                .cache_dir
                .as_ref()
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("XDG_CACHE_HOME")
                        .map(std::path::PathBuf::from)
                        .or_else(|| {
                            std::env::var_os("HOME")
                                .map(|home| std::path::PathBuf::from(home).join(".cache"))
                        })
                        .map(|p| p.join("sdev/review"))
                })
                .ok_or("Set --cache-dir when no user cache directory is available.")?;
            Some(live::Engine::new(
                &snapshot,
                language,
                args.jev_model.as_deref().unwrap_or("jev-1.13.0"),
                directory,
            )?)
        } else {
            None
        };
        // Pin the original base even if HEAD advances while reviewing the working tree.
        let watch = if args.input.is_none() && args.head.is_none() {
            let root = root.to_owned();
            let mut capture_args = args.clone();
            capture_args.base = Some(snapshot.base.clone());
            Some(Box::new(move || {
                let snapshot = capture(&root, &capture_args)?;
                validate(&snapshot)?;
                Ok(snapshot)
            }) as server::Capture)
        } else {
            None
        };
        return server::serve(
            snapshot,
            language,
            args.port.unwrap_or(0),
            live,
            watch,
            writer,
        );
    }
    let output = args.output.as_deref().unwrap_or(if args.prepare {
        "-"
    } else {
        "sdev-review.html"
    });
    if output == "-" {
        return writer
            .write_all(content.as_bytes())
            .map_err(|e| e.to_string());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("Cannot create {output} (existing files are preserved): {e}"))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Cannot write {output}: {e}"))?;
    writeln!(
        writer,
        "{} {output}",
        language.select("Created", "作成しました:")
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn markdown_retains_the_complete_source_range_for_inline_comments() {
        let rendered = super::markdown("# Title\n\nFirst sentence.\nSecond sentence.\n");
        assert!(rendered.contains("data-line=\"3\" data-line-end=\"4\""));
    }

    #[test]
    fn tables_keep_header_semantics_alignment_and_row_locations() {
        let rendered =
            super::markdown("# Title\n\n| 名前 | 数 |\n| --- | ---: |\n| `<tr>` | 2 |\n");
        assert!(rendered.contains("<thead><tr tabindex=\"0\" data-line=\"3\""));
        assert!(rendered.contains("<tr tabindex=\"0\" data-line=\"5\""));
        assert!(rendered.contains("<th>名前</th>"));
        assert!(rendered.contains("<td style=\"text-align: right\">2</td>"));
        assert!(rendered.contains("<code>&lt;tr&gt;</code>"));
        assert_eq!(rendered.matches("tabindex=\"0\" data-line=").count(), 3);
    }

    #[test]
    fn markdown_is_readable_but_cannot_load_resources_or_run_html() {
        let rendered = super::markdown(
            "# Title\n\n<script>alert(1)</script>\n\n[x](javascript:alert(1)) ![alt](https://example.invalid/pixel)\n",
        );
        assert!(rendered.contains("data-line=\"1\""));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("href="));
        assert!(!rendered.contains("<img"));
    }
}
