mod markdown;
mod paths;
mod report;

use markdown::{Markdown, Reference};
use paths::{Kind, Target};
pub use report::Report;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

struct Document {
    path: PathBuf,
    real: PathBuf,
    kind: Kind,
}
struct Checker {
    root: PathBuf,
    report: Report,
    cache: BTreeMap<PathBuf, Result<Markdown, (&'static str, String)>>,
}

pub fn check(root: &Path) -> Report {
    let mut checker = Checker {
        root: root.to_path_buf(),
        report: Report::default(),
        cache: BTreeMap::new(),
    };
    match root.canonicalize() {
        Ok(root) => checker.root = root,
        Err(error) => {
            checker.failure(
                Path::new(""),
                "discovery-failed",
                format!("ファイルを確認・読み取れません: {error}"),
            );
            return checker.report.finish();
        }
    }
    let mut targets = BTreeSet::new();
    for name in ["README.md", "AGENTS.md"] {
        let path = Path::new(name);
        match fs::symlink_metadata(checker.root.join(path)) {
            Ok(metadata) if metadata.is_dir() && !paths::unsafe_link(&metadata) => checker.failure(
                path,
                "discovery-failed",
                "入口文書が通常のファイルではありません".into(),
            ),
            Ok(metadata) => checker.discovered(path, metadata, &mut targets),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => checker.failure(
                path,
                "discovery-failed",
                format!("ファイルを確認・読み取れません: {error}"),
            ),
        }
    }
    // Root entries retain the same allowlist when localized; arbitrary root Markdown stays out of scope.
    match fs::read_dir(&checker.root) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = PathBuf::from(entry.file_name());
                        if path != Path::new("README.md")
                            && path != Path::new("AGENTS.md")
                            && [PathBuf::from("README.md"), PathBuf::from("AGENTS.md")]
                                .contains(&paths::document_identity(&path))
                        {
                            match fs::symlink_metadata(entry.path()) {
                                Ok(metadata)
                                    if metadata.is_dir() && !paths::unsafe_link(&metadata) =>
                                {
                                    checker.failure(
                                        &path,
                                        "discovery-failed",
                                        "入口文書が通常のファイルではありません".into(),
                                    )
                                }
                                Ok(metadata) => checker.discovered(&path, metadata, &mut targets),
                                Err(error) => {
                                    checker.failure(&path, "discovery-failed", error.to_string())
                                }
                            }
                        }
                    }
                    Err(error) => {
                        checker.failure(Path::new(""), "discovery-failed", error.to_string())
                    }
                }
            }
        }
        Err(error) => checker.failure(Path::new(""), "discovery-failed", error.to_string()),
    }
    let docs = Path::new("docs");
    match fs::symlink_metadata(checker.root.join(docs)) {
        Ok(metadata) if metadata.is_dir() && !paths::unsafe_link(&metadata) => {
            checker.discover(docs, &mut targets)
        }
        Ok(_) => checker.failure(
            docs,
            "discovery-failed",
            "docsが通常のディレクトリではありません".into(),
        ),
        Err(error) => checker.failure(
            docs,
            "discovery-failed",
            format!("ファイルを確認・読み取れません: {error}"),
        ),
    }
    if targets.is_empty() {
        checker.failure(
            docs,
            "discovery-failed",
            "検査対象のMarkdownがありません".into(),
        );
    }
    let mut documents = Vec::new();
    for path in targets {
        if path.to_str().is_none() {
            checker.failure(
                &path,
                "unsupported-path",
                "文書のパスがUTF-8ではありません".into(),
            );
            continue;
        }
        let real = match checker.boundary(&path) {
            Ok(real) => real,
            Err((code, reason)) => {
                checker.failure(&path, code, reason);
                continue;
            }
        };
        match checker.load(&real) {
            Ok(_) => {
                let kind = paths::kind(&path);
                checker.report.documents += 1;
                if kind.directional() {
                    checker.report.direction_documents += 1;
                }
                documents.push(Document { path, real, kind });
            }
            Err((code, reason)) => checker.failure(&path, code, reason),
        }
    }
    for document in documents {
        // Remove and restore so references can be checked while the cache is mutated.
        let parsed = checker.cache.remove(&document.real).unwrap().unwrap();
        for reference in &parsed.references {
            checker.reference(&document, reference, &parsed);
        }
        checker.cache.insert(document.real, Ok(parsed));
    }
    checker.report.finish()
}

impl Checker {
    fn failure(&mut self, path: &Path, code: &'static str, reason: String) {
        self.report.add(path, None, code, reason);
    }
    fn discover(&mut self, directory: &Path, targets: &mut BTreeSet<PathBuf>) {
        let entries = match fs::read_dir(self.root.join(directory)) {
            Ok(entries) => entries,
            Err(error) => {
                self.failure(
                    directory,
                    "discovery-failed",
                    format!("ファイルを確認・読み取れません: {error}"),
                );
                return;
            }
        };
        let mut entries: Vec<_> = entries
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry.path()),
                Err(error) => {
                    self.failure(
                        directory,
                        "discovery-failed",
                        format!("ファイルを確認・読み取れません: {error}"),
                    );
                    None
                }
            })
            .collect();
        entries.sort();
        for entry in entries {
            let path = entry.strip_prefix(&self.root).unwrap();
            match fs::symlink_metadata(&entry) {
                Ok(metadata) => self.discovered(path, metadata, targets),
                Err(error) => self.failure(
                    path,
                    "discovery-failed",
                    format!("ファイルを確認・読み取れません: {error}"),
                ),
            }
        }
    }
    fn discovered(&mut self, path: &Path, metadata: fs::Metadata, targets: &mut BTreeSet<PathBuf>) {
        if paths::unsafe_link(&metadata) {
            self.failure(
                path,
                "discovery-failed",
                "探索中のシンボリックリンク・再解析ポイントはたどりません".into(),
            );
        } else if metadata.is_dir() {
            self.discover(path, targets);
        } else if metadata.is_file() && paths::markdown(path) {
            targets.insert(path.to_path_buf());
        } else if paths::markdown(path) {
            self.failure(
                path,
                "discovery-failed",
                "通常ファイルではありません".into(),
            );
        }
    }
    // Resolve each existing component before probing a descendant. A missing file
    // under an external symlink must not be mistaken for an ordinary broken link.
    fn boundary(&self, path: &Path) -> Result<PathBuf, (&'static str, String)> {
        let mut probe = self.root.clone();
        for component in path.components() {
            probe.push(component);
            match probe.canonicalize() {
                Ok(real) if real.starts_with(&self.root) => {}
                Ok(_) => {
                    return Err((
                        "outside-root",
                        "参照先の実体がプロジェクトのルート外です".into(),
                    ));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Err(("missing-target", "参照先が存在しません".into()));
                }
                Err(error) => {
                    return Err((
                        "read-failed",
                        format!("ファイルを確認・読み取れません: {error}"),
                    ));
                }
            }
        }
        probe.canonicalize().map_err(|e| {
            (
                "read-failed",
                format!("ファイルを確認・読み取れません: {e}"),
            )
        })
    }
    fn load(&mut self, real: &Path) -> Result<&Markdown, (&'static str, String)> {
        if !self.cache.contains_key(real) {
            let result = (|| {
                let metadata = fs::metadata(real).map_err(|e| {
                    (
                        "read-failed",
                        format!("ファイルを確認・読み取れません: {e}"),
                    )
                })?;
                if !metadata.is_file() {
                    return Err(("read-failed", "参照先が通常のファイルではありません".into()));
                }
                let bytes = fs::read(real).map_err(|e| {
                    (
                        "read-failed",
                        format!("ファイルを確認・読み取れません: {e}"),
                    )
                })?;
                let text = String::from_utf8(bytes)
                    .map_err(|_| ("invalid-utf8", "文書がUTF-8ではありません".into()))?;
                Ok(markdown::parse(&text))
            })();
            self.cache.insert(real.to_path_buf(), result);
        }
        self.cache.get(real).unwrap().as_ref().map_err(Clone::clone)
    }
    fn reference(&mut self, source: &Document, reference: &Reference, parsed: &Markdown) {
        let resolved = paths::resolve(&source.path, &reference.target);
        if matches!(resolved, Ok(Target::External)) {
            self.report.external_references += 1;
            return;
        }
        self.report.internal_references += 1;
        let (path, fragment) = match resolved {
            Ok(Target::Local { path, fragment }) => (path, fragment),
            Err(error) => {
                self.report.add(
                    &source.path,
                    Some(reference),
                    error.code,
                    error.reason.into(),
                );
                return;
            }
            _ => unreachable!(),
        };
        let real = self.boundary(&path);
        let same = real.as_ref().is_ok_and(|real| *real == source.real);
        let same_document = match &real {
            Ok(target) => {
                paths::document_identity(target) == paths::document_identity(&source.real)
            }
            Err(("missing-target", _)) => {
                paths::document_identity(&path) == paths::document_identity(&source.path)
            }
            Err(_) => false,
        };
        let target_kind = paths::kind(
            real.as_ref()
                .ok()
                .and_then(|p| p.strip_prefix(&self.root).ok())
                .unwrap_or(&path),
        );
        if source.kind.directional()
            && !same_document
            && matches!(target_kind, Kind::Design | Kind::Implementation)
        {
            self.report.add(
                &source.path,
                Some(reference),
                "forbidden-reference",
                format!(
                    "{}から{}への参照は禁止です",
                    source.kind.label(),
                    if target_kind == Kind::Design {
                        "別文書のDesign"
                    } else {
                        "実装"
                    }
                ),
            );
        }
        let real = match real {
            Ok(real) => real,
            Err((code, reason)) => {
                self.report.add(&source.path, Some(reference), code, reason);
                return;
            }
        };
        let metadata = match fs::metadata(&real) {
            Ok(metadata) => metadata,
            Err(error) => {
                self.report.add(
                    &source.path,
                    Some(reference),
                    "read-failed",
                    format!("ファイルを確認・読み取れません: {error}"),
                );
                return;
            }
        };
        if source.kind.directional() && target_kind == Kind::Unknown && !metadata.is_dir() {
            self.report.add(
                &source.path,
                Some(reference),
                "unclassified-target",
                "参照方向の判定に必要な文書種別を特定できません".into(),
            );
        }
        if fragment.is_empty() {
            return;
        }
        if !paths::markdown(&real) || metadata.is_dir() {
            self.report.skipped_heading_checks += 1;
            return;
        }
        let contains = if same {
            Ok(parsed.anchors.contains(&fragment))
        } else {
            self.load(&real)
                .map(|parsed| parsed.anchors.contains(&fragment))
        };
        match contains {
            Ok(true) => {}
            Ok(false) => self.report.add(
                &source.path,
                Some(reference),
                "missing-heading",
                "参照先の見出しが存在しません".into(),
            ),
            Err((code, reason)) => self.report.add(&source.path, Some(reference), code, reason),
        }
    }
}
