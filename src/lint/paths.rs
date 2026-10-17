use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Product,
    Requirements,
    Design,
    Guideline,
    Document,
    Asset,
    Implementation,
    Unknown,
}
impl Kind {
    pub fn directional(self) -> bool {
        matches!(self, Self::Product | Self::Requirements | Self::Design)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Requirements => "Requirements",
            Self::Design => "Design",
            Self::Guideline => "ガイドライン",
            Self::Document => "一般文書",
            Self::Asset => "資産",
            Self::Implementation => "実装",
            Self::Unknown => "不明",
        }
    }
}
pub fn markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("md"))
}
pub fn kind(path: &Path) -> Kind {
    let parts: Vec<_> = path.iter().filter_map(|p| p.to_str()).collect();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if markdown(path) {
        match parts.as_slice() {
            ["docs", "products", _, "product.md"] => return Kind::Product,
            ["docs", "products", _, "features", _, "requirements.md"] => return Kind::Requirements,
            ["docs", "products", _, "features", _, "design.md"] => return Kind::Design,
            ["docs", "engineering", "design", ..] => return Kind::Design,
            ["docs", "guidelines", ..] | ["docs", "engineering", "guidelines", ..] => {
                return Kind::Guideline;
            }
            _ => return Kind::Document,
        }
    }
    if ["txt", "rst", "adoc"].contains(&ext.as_str())
        || matches!(parts.as_slice(), ["LICENSE" | "NOTICE"])
    {
        return Kind::Document;
    }
    if [
        "png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "avif", "pdf",
    ]
    .contains(&ext.as_str())
    {
        return Kind::Asset;
    }
    if parts.first().is_some_and(|p| {
        [
            "src", "lib", "bin", "tests", "xtask", "scripts", ".cargo", ".github",
        ]
        .contains(p)
    }) || [
        "rs", "js", "jsx", "mjs", "cjs", "ts", "tsx", "go", "py", "java", "kt", "kts", "swift",
        "c", "h", "cc", "cpp", "cxx", "hpp", "cs", "rb", "php", "sh", "bash", "zsh", "ps1", "sql",
    ]
    .contains(&ext.as_str())
        || matches!(
            parts.as_slice(),
            ["Cargo.toml"
                | "Cargo.lock"
                | "rust-toolchain.toml"
                | "package.json"
                | "package-lock.json"
                | "npm-shrinkwrap.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "bun.lock"
                | "bun.lockb"
                | ".gitignore"
                | ".gitattributes"
                | ".editorconfig"]
        )
    {
        Kind::Implementation
    } else {
        Kind::Unknown
    }
}

pub enum Target {
    External,
    Local { path: PathBuf, fragment: String },
}
pub struct Problem {
    pub code: &'static str,
    pub reason: &'static str,
}
fn problem(code: &'static str, reason: &'static str) -> Problem {
    Problem { code, reason }
}
fn decode(value: &str, path: bool) -> Result<String, Problem> {
    let bytes = value.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return Err(problem("invalid-target", "パーセント表現が不正です"));
        }
        if path && bytes[i] == b'%' && i + 2 < bytes.len() {
            let decoded = u8::from_str_radix(&value[i + 1..i + 3], 16).unwrap();
            if decoded == b'/' || decoded == b'\\' {
                return Err(problem(
                    "invalid-target",
                    "復号したパスに区切りが含まれます",
                ));
            }
        }
    }
    let result = percent_decode_str(value)
        .decode_utf8()
        .map_err(|_| problem("invalid-target", "参照先の符号化がUTF-8ではありません"))?
        .into_owned();
    if result.contains('\0') {
        return Err(problem("invalid-target", "参照先にNULが含まれます"));
    }
    Ok(result)
}
pub fn resolve(source: &Path, target: &str) -> Result<Target, Problem> {
    let bytes = target.as_bytes();
    if target.starts_with("\\\\")
        || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
    {
        return Err(problem(
            "unsupported-path",
            "Windows固有のローカルパスは扱えません",
        ));
    }
    if target.starts_with("//")
        || target.split_once(':').is_some_and(|(scheme, _)| {
            scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        })
    {
        return Ok(Target::External);
    }
    let (file, fragment) = target.split_once('#').unwrap_or((target, ""));
    let file = file.split_once('?').map_or(file, |(path, _)| path);
    let decoded = decode(file, true)?;
    let fragment = decode(fragment, false)?;
    if decoded.contains('\\') {
        return Err(problem(
            "unsupported-path",
            "バックスラッシュを使うローカルパスは扱えません",
        ));
    }
    let mut path = if decoded.is_empty() {
        source.to_path_buf()
    } else if decoded.starts_with('/') {
        PathBuf::new()
    } else {
        source.parent().unwrap_or(Path::new("")).to_path_buf()
    };
    if !decoded.is_empty() {
        for part in Path::new(decoded.trim_start_matches('/')).components() {
            match part {
                Component::Normal(value) => path.push(value),
                Component::CurDir => {}
                Component::ParentDir => {
                    if !path.pop() {
                        return Err(problem(
                            "outside-root",
                            "参照先がプロジェクトのルート外です",
                        ));
                    }
                }
                _ => return Err(problem("unsupported-path", "扱えないローカルパスです")),
            }
        }
    }
    Ok(Target::Local { path, fragment })
}

// Lossless path display: invalid OS strings retain their bytes/code units.
pub fn display(path: &Path) -> String {
    path.components()
        .filter_map(|part| match part {
            Component::Normal(value) => Some(
                value
                    .to_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| escaped_os(value)),
            ),
            Component::ParentDir => Some("..".into()),
            Component::CurDir => None,
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
#[cfg(unix)]
fn escaped_os(value: &std::ffi::OsStr) -> String {
    use std::os::unix::ffi::OsStrExt;
    value
        .as_bytes()
        .iter()
        .map(|b| format!("\\x{b:02x}"))
        .collect()
}
#[cfg(windows)]
fn escaped_os(value: &std::ffi::OsStr) -> String {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().map(|c| format!("\\u{c:04x}")).collect()
}
#[cfg(not(any(unix, windows)))]
fn escaped_os(value: &std::ffi::OsStr) -> String {
    format!("{value:?}")
}

pub fn unsafe_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn os_independent_reference_boundaries() {
        let source = Path::new("docs/source.md");
        for target in [
            "C:/outside.md",
            "c:relative.md",
            "\\\\host\\share\\file.md",
            "a\\b.md",
        ] {
            assert!(
                matches!(
                    resolve(source, target),
                    Err(Problem {
                        code: "unsupported-path",
                        ..
                    })
                ),
                "{target}"
            );
        }
        for target in ["../..", "../../private.md", "/../private.md"] {
            assert!(
                matches!(
                    resolve(source, target),
                    Err(Problem {
                        code: "outside-root",
                        ..
                    })
                ),
                "{target}"
            );
        }
        for target in [
            "a%2fb.md", "a%5Cb.md", "%", "%F", "%xy", "%FF", "#%FF", "#%00",
        ] {
            assert!(
                matches!(
                    resolve(source, target),
                    Err(Problem {
                        code: "invalid-target",
                        ..
                    })
                ),
                "{target}"
            );
        }
        assert!(
            matches!(resolve(source, "#"), Ok(Target::Local { path, fragment }) if path == source && fragment.is_empty())
        );
        for target in [
            "https://example.invalid",
            "mailto:x@y.invalid",
            "//example.invalid",
            "file:///etc/passwd",
        ] {
            assert!(matches!(resolve(source, target), Ok(Target::External)));
        }
    }
    #[cfg(unix)]
    #[test]
    fn invalid_os_strings_are_lossless_in_reports() {
        use std::os::unix::ffi::OsStrExt;
        let path = Path::new(std::ffi::OsStr::from_bytes(b"docs/invalid-\xff.md"));
        assert_eq!(
            display(path),
            "docs/\\x69\\x6e\\x76\\x61\\x6c\\x69\\x64\\x2d\\xff\\x2e\\x6d\\x64"
        );
    }
}
