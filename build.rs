use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Component, Path, PathBuf},
};

#[path = "src/catalog.rs"]
mod catalog;

fn markdown_files(directory: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(directory).expect("read material directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            files.extend(markdown_files(&path));
        } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "md") {
            files.insert(path);
        }
    }
    files
}

fn validate_catalog(root: &Path) {
    let mut names = BTreeSet::new();
    for topic in catalog::TOPICS {
        assert!(matches!(topic.operation, "help" | "template"));
        assert!(
            names.insert((topic.operation, topic.name)),
            "Duplicate topic: {} {}",
            topic.operation,
            topic.name
        );
    }
    for (operation, directory) in [("help", "docs/guidelines"), ("template", "docs/templates")] {
        println!("cargo:rerun-if-changed={directory}");
        let registered: BTreeSet<_> = catalog::TOPICS
            .iter()
            .filter(|topic| topic.operation == operation)
            .flat_map(|topic| {
                [
                    root.join(topic.source),
                    root.join(topic.source.replace(".md", ".ja.md")),
                ]
            })
            .collect();
        assert_eq!(
            registered,
            markdown_files(&root.join(directory)),
            "Catalog must match Markdown files under {directory}"
        );
    }
}

fn target_command(source: &str, destination: &str) -> Option<String> {
    if destination.contains("://") || destination.starts_with("mailto:") {
        return None;
    }
    let (file, anchor) = destination.split_once('#').unwrap_or((destination, ""));
    let joined = if file.is_empty() {
        PathBuf::from(source)
    } else {
        Path::new(source).parent().unwrap().join(file)
    };
    let mut path = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                path.pop();
            }
            Component::CurDir => {}
            _ => path.push(component.as_os_str()),
        }
    }
    let normalized = path.to_str().unwrap().replace('\\', "/");
    let japanese = normalized.ends_with(".ja.md");
    let normalized = normalized.replace(".ja.md", ".md");
    let command = if normalized == "docs/README.md" {
        "sdev help".to_owned()
    } else {
        let topic = catalog::TOPICS
            .iter()
            .find(|topic| topic.source == normalized)
            .unwrap_or_else(|| panic!("Unmapped guide link: {source} -> {destination}"));
        format!("sdev {} {}", topic.operation, topic.name)
    };
    let command = format!("{command} --lang {}", if japanese { "ja" } else { "en" });
    Some(if anchor.is_empty() {
        format!("`{command}`")
    } else {
        if japanese {
            format!("`{command}`の「{anchor}」節")
        } else {
            format!("`{command}` (section: {anchor})")
        }
    })
}

fn guide_body(source: &str, text: &str) -> String {
    let mut edits = Vec::new();
    let mut link: Option<(usize, String, String)> = None;
    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(command) = target_command(source, &dest_url) {
                    link = Some((range.start, command, String::new()));
                }
            }
            Event::Code(text) => {
                if let Some((_, _, label)) = &mut link {
                    label.push_str(&text);
                } else if (text.as_ref() == "sdev help"
                    || text.starts_with("sdev help ")
                    || text.as_ref() == "sdev template"
                    || text.starts_with("sdev template "))
                    && !text.contains("--lang")
                {
                    let language = if source.ends_with(".ja.md") {
                        "ja"
                    } else {
                        "en"
                    };
                    edits.push((range, format!("`{text} --lang {language}`")));
                }
            }
            Event::Text(text) => {
                if let Some((_, _, label)) = &mut link {
                    label.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, _, label)) = &mut link {
                    label.push(' ');
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((start, command, label)) = link.take() {
                    edits.push((
                        start..range.end,
                        if source.ends_with(".ja.md") {
                            format!("{label}（{command}）")
                        } else {
                            format!("{label} ({command})")
                        },
                    ));
                }
            }
            _ => {}
        }
    }
    let mut result = text.to_owned();
    for (range, replacement) in edits.into_iter().rev() {
        result.replace_range(range, &replacement);
    }
    result
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-changed=src/catalog.rs");
    validate_catalog(&root);
    let mut generated = String::from("const MATERIALS: &[Material] = &[\n");
    for topic in catalog::TOPICS {
        for (language, source, title) in [
            ("en", topic.source.to_owned(), topic.title),
            ("ja", topic.source.replace(".md", ".ja.md"), topic.title_ja),
        ] {
            println!("cargo:rerun-if-changed={source}");
            let original = fs::read_to_string(root.join(&source)).expect("read UTF-8 Markdown");
            let body = if topic.operation == "help" {
                guide_body(&source, &original)
            } else {
                original
            };
            generated.push_str(&format!(
                "Material {{ operation: {:?}, topic: {:?}, language: {:?}, title: {:?}, body: {:?} }},\n",
                topic.operation, topic.name, language, title, body
            ));
        }
    }
    generated.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("materials.rs"),
        generated,
    )
    .unwrap();
}
