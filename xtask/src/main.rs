mod benchmark;
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
                "cargo xtask check [--target TARGET]\ncargo xtask build-release --target TARGET\ncargo xtask verify-release BINARY\ncargo xtask measure [--rust]"
            );
            Ok(())
        }
        ["check"] => check(None),
        ["check", "--target", target] => check(Some(target)),
        ["build-release", "--target", target] => release::build(target),
        ["verify-release", binary] => release::verify(Path::new(binary)),
        ["measure"] => benchmark::measure(false),
        ["measure", "--rust"] => benchmark::measure(true),
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
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
