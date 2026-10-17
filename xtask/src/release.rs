use crate::support::{self, Result, TemporaryDirectory, Toolchain, root};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::Write,
    path::Path,
    process::{Command, Output},
    time::Duration,
};

#[path = "../../src/catalog.rs"]
mod catalog;

pub struct Target {
    pub name: &'static str,
    pub label: &'static str,
    pub runner: &'static str,
    os: &'static str,
    arch: &'static str,
    minimum: Option<&'static str>,
}
pub const TARGETS: &[Target] = &[
    Target {
        name: "aarch64-apple-darwin",
        label: "macos-arm64",
        runner: "macos-15",
        os: "macos",
        arch: "aarch64",
        minimum: Some("11.0"),
    },
    Target {
        name: "x86_64-apple-darwin",
        label: "macos-x64",
        runner: "macos-15-intel",
        os: "macos",
        arch: "x86_64",
        minimum: Some("10.12"),
    },
    Target {
        name: "aarch64-unknown-linux-gnu",
        label: "linux-arm64",
        runner: "ubuntu-22.04-arm",
        os: "linux",
        arch: "aarch64",
        minimum: None,
    },
    Target {
        name: "x86_64-unknown-linux-gnu",
        label: "linux-x64",
        runner: "ubuntu-22.04",
        os: "linux",
        arch: "x86_64",
        minimum: None,
    },
    Target {
        name: "x86_64-pc-windows-msvc",
        label: "windows-x64",
        runner: "windows-2025",
        os: "windows",
        arch: "x86_64",
        minimum: None,
    },
];
fn configuration(name: &str) -> Result<&'static Target> {
    TARGETS
        .iter()
        .find(|target| target.name == name)
        .ok_or_else(|| format!("Unsupported target: {name}").into())
}
pub fn matrix() {
    let include: Vec<_> = TARGETS
        .iter()
        .map(|target| json!({"target": target.name, "runner": target.runner}))
        .collect();
    println!("{}", json!({"include": include}));
}
pub fn asset_name(version: &str, target: &Target) -> String {
    format!(
        "sdev-v{version}-{}{}",
        target.label,
        if target.os == "windows" { ".exe" } else { "" }
    )
}
pub fn source_commit() -> Result<String> {
    Ok(support::output(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root()),
    )?
    .trim()
    .to_owned())
}

pub fn target(name: &str) -> Result<()> {
    configuration(name).map(|_| ())
}

pub fn build(name: &str) -> Result<()> {
    let target = configuration(name)?;
    let toolchain = Toolchain::new()?;
    let metadata: Value = serde_json::from_str(&support::output(toolchain.cargo().args([
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
        .find(|package| package["name"] == "sdev")
        .ok_or("Missing sdev package")?;
    let version = package["version"].as_str().ok_or("Missing version")?;
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
    {
        return Err("Unsupported release version".into());
    }
    support::run(Command::new("rustup").args([
        "target",
        "add",
        "--toolchain",
        &toolchain.channel,
        name,
    ]))?;
    let mut command = toolchain.cargo();
    command.args([
        "build",
        "--locked",
        "--release",
        "--package",
        "sdev",
        "--target",
        name,
    ]);
    if let Some(minimum) = target.minimum {
        command.env("MACOSX_DEPLOYMENT_TARGET", minimum);
    }
    support::run(&mut command)?;
    let extension = if target.os == "windows" { ".exe" } else { "" };
    let source = Path::new(
        metadata["target_directory"]
            .as_str()
            .ok_or("Missing target directory")?,
    )
    .join(name)
    .join("release")
    .join(format!("sdev{extension}"));
    let asset = asset_name(version, target);
    let relative = format!("dist/{asset}");
    let artifact = root().join(&relative);
    fs::create_dir_all(artifact.parent().unwrap())?;
    fs::copy(source, &artifact)?;
    let bytes = fs::read(&artifact)?;
    let digest = support::digest(&bytes);
    fs::write(
        support::sidecar(&artifact, ".sha256"),
        format!("{digest}  {asset}\n"),
    )?;
    let notices = "THIRD-PARTY-NOTICES.html";
    fs::copy(
        root().join(format!(
            "licenses/rust-{}-COPYRIGHT-library.html",
            toolchain.channel
        )),
        artifact.parent().unwrap().join(notices),
    )?;
    let crate_notices = "THIRD-PARTY-CRATES.html";
    fs::write(
        artifact.parent().unwrap().join(crate_notices),
        crate::notices::crates(name)?,
    )?;
    if root().join("LICENSE").is_file() {
        fs::copy(
            root().join("LICENSE"),
            artifact.parent().unwrap().join("LICENSE"),
        )?;
    }
    let info = json!({"version":version,"target":name,"binary":relative,"bytes":bytes.len(),"sha256":digest,"published":false,"notices":notices,"crate_notices":crate_notices,"source_commit":source_commit()?,"source_clean":support::output(Command::new("git").args(["status", "--porcelain"]).current_dir(root()))?.trim().is_empty(),"verified":false});
    fs::write(
        support::sidecar(&artifact, ".json"),
        format!("{}\n", serde_json::to_string_pretty(&info)?),
    )?;
    if let Some(output) = env::var_os("GITHUB_OUTPUT") {
        let mut file = fs::OpenOptions::new().append(true).open(output)?;
        writeln!(file, "binary={relative}\nasset={asset}\nversion={version}")?;
    }
    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())
}

fn invoke(binary: &Path, directory: &Path, args: &[&str], status: i32) -> Result<Output> {
    let mut command = Command::new(binary);
    command
        .args(args)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "");
    for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    let result = support::bounded_output(&mut command, Duration::from_secs(10))?;
    if result.status.code() != Some(status) || (status == 0 && !result.stderr.is_empty()) {
        return Err(format!(
            "{args:?}: {}: {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(result)
}

pub fn verify(path: &Path) -> Result<()> {
    let binary = path.canonicalize()?;
    let metadata: Value = serde_json::from_slice(&fs::read(support::sidecar(&binary, ".json"))?)?;
    let bytes = fs::read(&binary)?;
    let digest = support::digest(&bytes);
    let checksum = fs::read_to_string(support::sidecar(&binary, ".sha256"))?;
    if metadata["sha256"] != digest
        || checksum.split_whitespace().next() != Some(&digest)
        || metadata["bytes"].as_u64() != Some(bytes.len() as u64)
        || metadata["published"] != false
    {
        return Err("Release checksum or metadata mismatch".into());
    }
    let target_name = metadata["target"].as_str().ok_or("Missing target")?;
    let target = configuration(target_name)?;
    if target.os != env::consts::OS || target.arch != env::consts::ARCH {
        return Err("Run this check natively on the target OS and CPU".into());
    }
    let directory = TemporaryDirectory::new("release")?;
    let version = metadata["version"].as_str().ok_or("Missing version")?;
    if invoke(&binary, &directory.0, &["--version"], 0)?.stdout
        != format!("sdev {version}\n").as_bytes()
    {
        return Err("Release version mismatch".into());
    }
    let help = String::from_utf8(invoke(&binary, &directory.0, &["--help"], 0)?.stdout)?;
    for language in ["en", "ja"] {
        let localized_help = String::from_utf8(
            invoke(&binary, &directory.0, &["help", "--lang", language], 0)?.stdout,
        )?;
        let templates = String::from_utf8(
            invoke(&binary, &directory.0, &["template", "--lang", language], 0)?.stdout,
        )?;
        for topic in catalog::TOPICS {
            let overview = if topic.operation == "help" {
                &localized_help
            } else {
                &templates
            };
            if !overview.contains(&format!("sdev {} {}", topic.operation, topic.name))
                || !overview.contains(if language == "en" {
                    topic.title
                } else {
                    topic.title_ja
                })
            {
                return Err(format!(
                    "Missing topic in overview: {} {}",
                    topic.operation, topic.name
                )
                .into());
            }
            let body = invoke(
                &binary,
                &directory.0,
                &[topic.operation, topic.name, "--lang", language],
                0,
            )?
            .stdout;
            if topic.operation == "template" {
                let source = if language == "en" {
                    topic.source.to_owned()
                } else {
                    topic.source.replace(".md", ".ja.md")
                };
                if body != fs::read(root().join(source))? {
                    return Err(format!("Template body mismatch: {}", topic.name).into());
                }
            } else if !body.starts_with(b"# ") {
                return Err(format!("Invalid guide body: {}", topic.name).into());
            }
        }
    }
    for removed in ["docs get", "entry", "licenses", "docs/guidelines/"] {
        if help.contains(removed) {
            return Err(format!("Unexpected public operation: {removed}").into());
        }
    }
    for args in [
        &["help", "nonexistent"][..],
        &["licenses"],
        &["docs", "get", "entry"],
    ] {
        let result = invoke(&binary, &directory.0, args, 2)?;
        if !result.stdout.is_empty() || result.stderr.is_empty() {
            return Err(format!("Invalid diagnostic: {args:?}").into());
        }
    }
    let notices = binary.parent().unwrap().join("THIRD-PARTY-NOTICES.html");
    if metadata["notices"] != "THIRD-PARTY-NOTICES.html"
        || fs::read(notices)?
            != fs::read(root().join(format!(
                "licenses/rust-{}-COPYRIGHT-library.html",
                Toolchain::new()?.channel
            )))?
    {
        return Err("Rust notices mismatch".into());
    }
    if metadata["crate_notices"] != "THIRD-PARTY-CRATES.html"
        || fs::read_to_string(binary.parent().unwrap().join("THIRD-PARTY-CRATES.html"))?
            != crate::notices::crates(target_name)?
    {
        return Err("Crate notices mismatch".into());
    }
    let absent = invoke(&binary, &directory.0, &["lint", "--format", "json"], 2)?;
    let absent: Value = serde_json::from_slice(&absent.stdout)?;
    if absent["status"] != "incomplete" {
        return Err("Missing lint discovery failure".into());
    }
    if fs::read_dir(&directory.0)?.next().is_some() {
        return Err("CLI created files".into());
    }
    fs::create_dir(directory.0.join("docs"))?;
    fs::write(
        directory.0.join("docs/guide.md"),
        "# Guide\n[self](#guide)\n",
    )?;
    let pass = invoke(&binary, &directory.0, &["lint", "--format", "json"], 0)?;
    let pass: Value = serde_json::from_slice(&pass.stdout)?;
    if pass["status"] != "pass" || pass["summary"]["internal_references"] != 1 {
        return Err("Isolated lint failed".into());
    }
    fs::write(directory.0.join("docs/guide.md"), "[missing](missing.md)\n")?;
    let violations = invoke(&binary, &directory.0, &["lint"], 1)?;
    if !violations.stderr.is_empty()
        || !String::from_utf8(violations.stdout)?.contains("[missing-target]")
    {
        return Err("Missing lint violation".into());
    }
    invoke(&binary, &directory.0, &["init", "--help"], 0)?;
    invoke(&binary, &directory.0, &["init"], 0)?;
    let instructions = fs::read(directory.0.join("AGENTS.md"))?;
    let ignores = fs::read(directory.0.join(".gitignore"))?;
    let languages = fs::read(directory.0.join("sdev.toml"))?;
    if languages != b"language = \"en\"\ntranslations = []\n" {
        return Err("Unexpected initialized document languages".into());
    }
    invoke(&binary, &directory.0, &["init"], 0)?;
    if instructions != fs::read(directory.0.join("AGENTS.md"))?
        || ignores != fs::read(directory.0.join(".gitignore"))?
        || languages != fs::read(directory.0.join("sdev.toml"))?
    {
        return Err("Repeated initialization changed existing project instructions".into());
    }
    library_checks(&binary, target)?;
    if metadata["source_commit"] != source_commit()? {
        return Err("Artifact source commit mismatch".into());
    }
    let mut metadata = metadata;
    metadata["verified"] = json!(true);
    fs::write(
        support::sidecar(&binary, ".json"),
        format!("{}\n", serde_json::to_string_pretty(&metadata)?),
    )?;
    println!(
        "Verified {}: native execution, exact templates, guide routing, empty PATH/cwd, checksum, system libraries",
        binary.display()
    );
    Ok(())
}

fn macos_libraries(listing: &str) -> Result<()> {
    for line in listing
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
    {
        let path = line
            .split_whitespace()
            .next()
            .ok_or("Missing library path")?;
        if !path.starts_with("/usr/lib/") && !path.starts_with("/System/Library/") {
            return Err(format!("Non-system library dependency: {path}").into());
        }
    }
    Ok(())
}

fn macos_minimum(commands: &str, minimum: &str) -> Result<()> {
    let mut field = None;
    for line in commands.lines().map(str::trim) {
        if line.starts_with("cmd ") {
            field = match line {
                "cmd LC_BUILD_VERSION" => Some("minos "),
                "cmd LC_VERSION_MIN_MACOSX" => Some("version "),
                _ => None,
            };
        }
        if let Some(value) = field.and_then(|field| line.strip_prefix(field)) {
            return if value == minimum {
                Ok(())
            } else {
                Err("Unexpected macOS deployment target".into())
            };
        }
    }
    Err("Missing macOS deployment target".into())
}

fn linux_libraries(versions: &str) -> Result<()> {
    let versions: Vec<Vec<u32>> = versions
        .split("GLIBC_")
        .skip(1)
        .filter_map(|suffix| {
            let value: String = suffix
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if value.is_empty() {
                None
            } else {
                Some(
                    value
                        .split('.')
                        .map(str::parse)
                        .collect::<std::result::Result<_, _>>(),
                )
            }
        })
        .collect::<std::result::Result<_, _>>()?;
    if versions.is_empty()
        || versions
            .iter()
            .any(|version| version.as_slice() > [2, 35].as_slice())
    {
        return Err(format!("Unexpected glibc requirements: {versions:?}").into());
    }
    Ok(())
}

fn windows_libraries(listing: &str) -> Result<()> {
    if listing
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .any(|name| {
            name.ends_with(".dll")
                && ["vcruntime", "msvcp", "msvcr", "concrt"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
        })
    {
        return Err("Additional C runtime DLL dependency".into());
    }
    Ok(())
}

fn library_checks(binary: &Path, target: &Target) -> Result<()> {
    match target.os {
        "macos" => {
            macos_libraries(&support::output(
                Command::new("otool").arg("-L").arg(binary),
            )?)?;
            macos_minimum(
                &support::output(Command::new("otool").arg("-l").arg(binary))?,
                target.minimum.unwrap(),
            )
        }
        "linux" => linux_libraries(&support::output(
            Command::new("readelf").arg("--version-info").arg(binary),
        )?),
        "windows" => {
            let installer =
                Path::new(&env::var_os("ProgramFiles(x86)").ok_or("Missing ProgramFiles(x86)")?)
                    .join("Microsoft Visual Studio/Installer/vswhere.exe");
            let candidates = support::output(Command::new(installer).args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-find",
                "VC/Tools/MSVC/**/bin/Hostx64/x64/dumpbin.exe",
            ]))?;
            let dumpbin = candidates
                .lines()
                .next()
                .ok_or("dumpbin is required to inspect DLL dependencies")?;
            windows_libraries(&support::output(
                Command::new(dumpbin).arg("/dependents").arg(binary),
            )?)
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_extra_libraries_and_wrong_minimums() {
        assert!(macos_libraries("sdev:\n /usr/lib/libSystem.B.dylib (version)\n").is_ok());
        assert!(macos_libraries("sdev:\n /opt/homebrew/lib/libextra.dylib (version)\n").is_err());
        assert!(macos_minimum(" cmd LC_BUILD_VERSION\n minos 11.0\n sdk 15.0", "11.0").is_ok());
        assert!(macos_minimum(" cmd LC_VERSION_MIN_MACOSX\n version 10.12", "10.12").is_ok());
        assert!(macos_minimum(" cmd LC_BUILD_VERSION\n minos 15.0", "11.0").is_err());
        assert!(macos_minimum(" cmd LC_UUID", "11.0").is_err());
        assert!(linux_libraries("GLIBC_2.17 GLIBC_2.35").is_ok());
        assert!(linux_libraries("GLIBC_2.36").is_err());
        assert!(linux_libraries("").is_err());
        assert!(windows_libraries("KERNEL32.dll\nADVAPI32.dll").is_ok());
        for dll in [
            "VCRUNTIME140.dll",
            "MSVCP140.dll",
            "msvcr120.dll",
            "concrt140.dll",
        ] {
            assert!(windows_libraries(dll).is_err());
        }
    }

    #[test]
    fn rejects_damaged_executable_before_running_it() {
        let directory = TemporaryDirectory::new("damaged-artifact").unwrap();
        let binary = directory.0.join("sdev-v0.1.0-macos-arm64");
        fs::write(&binary, b"damaged executable").unwrap();
        fs::write(
            support::sidecar(&binary, ".sha256"),
            "original-checksum  sdev\n",
        )
        .unwrap();
        fs::write(
            support::sidecar(&binary, ".json"),
            br#"{"sha256":"original-checksum","bytes":18,"published":false}"#,
        )
        .unwrap();
        assert!(
            verify(&binary)
                .unwrap_err()
                .to_string()
                .contains("checksum")
        );
    }
}
