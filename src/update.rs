use crate::language::Language;
use std::{collections::BTreeMap, env};

pub fn arguments(values: &[&str]) -> Result<Vec<String>, String> {
    let mut options = BTreeMap::new();
    let mut values = values.iter();
    while let Some(&key) = values.next() {
        let value = match key {
            "--verify" => "",
            "--version" | "--path" | "--source-commit" | "--trusted-root" => *values
                .next()
                .ok_or_else(|| format!("{key} requires a value."))?,
            _ => return Err(format!("Unknown update argument: {key}")),
        };
        if key != "--verify" && value.is_empty() {
            return Err(format!("{key} requires a value."));
        }
        if options.insert(key, value.to_owned()).is_some() {
            return Err(format!("Specify {key} only once."));
        }
    }
    match env::var("SDEV_VERIFY").as_deref() {
        Err(env::VarError::NotPresent) | Ok("0") => {}
        Ok("1") => {
            options.insert("--verify", String::new());
        }
        _ => return Err("SDEV_VERIFY must be 0 or 1.".into()),
    }
    for key in ["--source-commit", "--trusted-root"] {
        if options.contains_key(key) && !options.contains_key("--verify") {
            return Err(format!("{key} requires --verify."));
        }
    }
    if let Some(value) = options.get("--source-commit")
        && (value.len() != 40
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')))
    {
        return Err("Source commit must be a 40-character lowercase SHA.".into());
    }
    if let Some(value) = options.get("--version") {
        let parts: Vec<_> = value.split('.').collect();
        if parts.len() != 3
            || parts.iter().any(|part| {
                part.is_empty()
                    || !part.bytes().all(|b| b.is_ascii_digit())
                    || part.len() > 1 && part.starts_with('0')
            })
        {
            return Err("Version must be X.Y.Z without v.".into());
        }
    }
    let path = match options.remove("--path") {
        Some(path) => path,
        None => match env::var("SDEV_LAUNCHER_PATH") {
            Ok(path) => path,
            Err(env::VarError::NotPresent) => "bin/sdev".to_owned(),
            Err(_) => return Err("SDEV_LAUNCHER_PATH must be UTF-8.".into()),
        },
    };
    #[cfg(unix)]
    let path = crate::launcher::normalize_path(&path)?;
    let mut arguments = vec!["--update".to_owned(), "--path".to_owned(), path];
    for (key, value) in options {
        arguments.push(key.to_owned());
        if key != "--verify" {
            arguments.push(value);
        }
    }
    Ok(arguments)
}

#[cfg(unix)]
pub fn run(arguments: &[String], language: Language) -> Result<String, String> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new("sh")
        .args(["-s", "--"])
        .args(arguments)
        .env("SDEV_UPDATE_LANGUAGE", language.code())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("Cannot start project update: {error}"))?;
    if let Err(error) = child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(include_bytes!("../install.sh"))
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("Cannot prepare project update: {error}"));
    }
    let result = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !result.status.success() {
        return Err(language
            .select(
                "Project update failed; the existing launcher was preserved.",
                "更新に失敗しました。既存の起動スクリプトは保持されています。",
            )
            .into());
    }
    String::from_utf8(result.stdout).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
pub fn run(_arguments: &[String], language: Language) -> Result<String, String> {
    Err(language.select("Project update supports macOS and Linux launchers. Update other installations through their installation method.", "起動スクリプトの更新はmacOS・Linuxに対応しています。ほかの導入方法では、その方法で更新してください。").into())
}
