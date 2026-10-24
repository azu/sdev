use crate::support::{self, Result, Toolchain};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Use the normal dependency graph for every target so the release has one
// shared notice file. License bodies come from the exact package in Cargo.lock.
pub fn crates() -> Result<String> {
    let toolchain = Toolchain::new()?;
    let tree = support::output(toolchain.cargo().args([
        "tree",
        "--locked",
        "--package",
        "sdev",
        "--edges",
        "normal",
        "--prefix",
        "none",
        "--format",
        "{p}",
        "--target",
        "all",
    ]))?;
    let metadata: Value = serde_json::from_str(&support::output(toolchain.cargo().args([
        "metadata",
        "--locked",
        "--format-version",
        "1",
    ]))?)?;
    let packages = metadata["packages"].as_array().ok_or("Missing packages")?;
    let selected: BTreeSet<_> = tree
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            Some((words.next()?, words.next()?.strip_prefix('v')?))
        })
        .filter(|(name, _)| *name != "sdev")
        .collect();
    let mut html = "<!doctype html><meta charset=\"utf-8\"><title>sdev third-party crates</title>\n<h1>Third-party crates included in sdev</h1>\n".to_owned();
    for (name, version) in selected {
        let package = packages
            .iter()
            .find(|p| p["name"] == name && p["version"] == version)
            .ok_or("Missing selected crate metadata")?;
        let manifest = Path::new(
            package["manifest_path"]
                .as_str()
                .ok_or("Missing manifest path")?,
        );
        let directory = manifest.parent().unwrap();
        let mut licenses = Vec::new();
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            let name = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_ascii_uppercase();
            if path.is_file()
                && (name.starts_with("LICENSE")
                    || name.starts_with("COPYING")
                    || name.starts_with("NOTICE")
                    || name == "UNLICENSE")
            {
                licenses.push(path);
            }
        }
        licenses.sort();
        if licenses.is_empty() {
            return Err(format!("No license material for {name} {version}").into());
        }
        html.push_str(&format!(
            "<h2>{} {}</h2>\n<p>{}</p>\n",
            escape(name),
            escape(version),
            escape(
                package["license"]
                    .as_str()
                    .ok_or("Missing license declaration")?
            )
        ));
        for path in licenses {
            html.push_str(&format!(
                "<h3>{}</h3>\n<pre>{}</pre>\n",
                escape(
                    path.file_name()
                        .unwrap()
                        .to_str()
                        .ok_or("Invalid license filename")?
                ),
                escape(&fs::read_to_string(path)?)
            ));
        }
    }
    Ok(html)
}

#[cfg(test)]
mod tests {
    #[test]
    fn notice_text_cannot_become_html() {
        assert_eq!(super::escape("<&>\""), "&lt;&amp;&gt;&quot;");
    }
}
