use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process,
    time::Instant,
};

include!(concat!(env!("OUT_DIR"), "/documents.rs"));

fn help() {
    println!(
        "Comparison prototype: --help | --version | docs [name] | check <markdown>... | profile <markdown>..."
    );
}

fn extract_links(text: &str) -> Vec<String> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    Parser::new_ext(text, options)
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                Some(dest_url.into_string())
            }
            _ => None,
        })
        .collect()
}

fn is_external(href: &str) -> bool {
    if href.starts_with('#') || href.starts_with("//") {
        return true;
    }
    if let Some((scheme, _)) = href.split_once(':') {
        let mut chars = scheme.chars();
        return chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'));
    }
    false
}

fn resolve_path(file: &str, relative: &str) -> Result<PathBuf, String> {
    let decoded = percent_decode_str(relative)
        .decode_utf8()
        .map_err(|e| e.to_string())?;
    let parent = Path::new(file).parent().unwrap_or_else(|| Path::new("."));
    let full = env::current_dir()
        .map_err(|e| e.to_string())?
        .join(parent)
        .join(decoded.as_ref());
    let mut normalized = PathBuf::new();
    for component in full.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            _ => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

fn inspect_links(file: &str, links: &[String]) -> Result<(Vec<Value>, usize), String> {
    let mut records = Vec::new();
    let mut missing = 0;
    for href in links {
        if is_external(href) {
            continue;
        }
        let part = href.split('#').next().unwrap();
        if part.is_empty() {
            continue;
        }
        let target = resolve_path(file, part)?;
        let exists = target.exists();
        if !exists {
            missing += 1;
        }
        records.push(json!({"file": file, "href": href, "target": target.to_string_lossy(), "exists": exists}));
    }
    Ok((records, missing))
}

fn read_text(file: &str) -> Result<String, String> {
    fs::read_to_string(file).map_err(|error| format!("{file}: {error}"))
}

fn main_result() -> Result<i32, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first() else {
        help();
        return Ok(0);
    };
    match command.as_str() {
        "--help" => {
            help();
            Ok(0)
        }
        "--version" => {
            println!("{VERSION}");
            Ok(0)
        }
        "docs" => {
            if args.len() < 2 {
                for (name, _) in DOCUMENTS {
                    println!("{name}");
                }
                return Ok(0);
            }
            if let Some((_, body)) = DOCUMENTS.iter().find(|(name, _)| *name == args[1]) {
                println!("{body}");
                Ok(0)
            } else {
                Err(format!("Unknown document: {}; use docs to list", args[1]))
            }
        }
        "check" if args.len() > 1 => {
            let mut missing = 0;
            for file in &args[1..] {
                let text = read_text(file)?;
                let links = extract_links(&text);
                let (records, count) = inspect_links(file, &links)?;
                missing += count;
                for record in records {
                    println!("{record}");
                }
            }
            Ok(if missing > 0 { 1 } else { 0 })
        }
        "profile" if args.len() > 1 => {
            let (mut read_ms, mut parse_ms, mut references_ms) = (0.0, 0.0, 0.0);
            let (mut links_count, mut missing) = (0, 0);
            for file in &args[1..] {
                let t0 = Instant::now();
                let text = read_text(file)?;
                let t1 = Instant::now();
                let links = extract_links(&text);
                let t2 = Instant::now();
                let (records, count) = inspect_links(file, &links)?;
                let t3 = Instant::now();
                read_ms += (t1 - t0).as_secs_f64() * 1000.0;
                parse_ms += (t2 - t1).as_secs_f64() * 1000.0;
                references_ms += (t3 - t2).as_secs_f64() * 1000.0;
                links_count += records.len();
                missing += count;
            }
            println!(
                "{}",
                json!({"read_ms":read_ms,"parse_ms":parse_ms,"references_ms":references_ms,"total_ms":read_ms+parse_ms+references_ms,"files":args.len()-1,"links":links_count,"missing":missing})
            );
            Ok(if missing > 0 { 1 } else { 0 })
        }
        _ => Err("Unknown command or missing input; use --help".to_string()),
    }
}

fn main() {
    match main_result() {
        Ok(status) => process::exit(status),
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    }
}
