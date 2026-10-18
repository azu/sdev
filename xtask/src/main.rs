mod evals;
mod notices;
mod publishing;
mod release;
mod support;

use std::{env, path::Path};
use support::{Result, Toolchain};

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["--help" | "-h"] => {
            println!(
                "cargo xtask check [--target TARGET]\ncargo xtask eval prepare --binary BINARY --output DIRECTORY\ncargo xtask eval judge-prepare --output DIRECTORY\ncargo xtask eval score --output DIRECTORY\ncargo xtask build-release --target TARGET\ncargo xtask verify-release BINARY\ncargo xtask release-matrix\ncargo xtask release-preflight VERSION\ncargo xtask stage-release VERSION ARTIFACTS ATTESTATION"
            );
            Ok(())
        }
        ["release-matrix"] => {
            release::matrix();
            Ok(())
        }
        ["eval", arguments @ ..] => evals::run(arguments),
        ["release-preflight", version] => publishing::preflight(version),
        ["stage-release", version, artifacts, output] => {
            publishing::stage(version, Path::new(artifacts), Path::new(output))
        }
        ["publish-release", version, artifacts, attestation] => {
            publishing::run(version, Path::new(artifacts), Path::new(attestation))
        }
        ["check"] => check(None),
        ["check", "--target", target] => check(Some(target)),
        ["build-release", "--target", target] => release::build(target),
        ["verify-release", binary] => release::verify(Path::new(binary)),
        _ => Err("Unknown task or arguments; use cargo xtask --help".into()),
    }
}

fn check(target: Option<&str>) -> Result<()> {
    let toolchain = Toolchain::new()?;
    support::run(toolchain.cargo().args(["fmt", "--all", "--", "--check"]))?;
    for task in ["test", "clippy"] {
        let mut command = toolchain.cargo();
        command.args([task, "--locked", "--workspace", "--all-targets"]);
        if let Some(target) = target {
            release::target(target)?;
            command.args(["--target", target]);
        }
        if task == "clippy" {
            command.args(["--", "-D", "warnings"]);
        }
        support::run(&mut command)?;
    }
    let mut lint = toolchain.cargo();
    lint.args(["run", "--locked", "--package", "sdev"]);
    if let Some(target) = target {
        lint.args(["--target", target]);
    }
    lint.args(["--", "lint"]);
    support::run(&mut lint)?;
    Ok(())
}

fn main() {
    if env::current_exe()
        .ok()
        .is_some_and(|path| path.file_stem().is_some_and(|name| name == "sdev"))
    {
        let arguments: Vec<String> = env::args().skip(1).collect();
        let code = evals::observe_wrapper(&arguments).unwrap_or_else(|error| {
            eprintln!("{error}");
            2
        });
        std::process::exit(code);
    }
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
