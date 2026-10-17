use std::{
    env,
    io::{self, Write},
    process::ExitCode,
};

struct Material {
    operation: &'static str,
    topic: &'static str,
    title: &'static str,
    body: &'static str,
}
include!(concat!(env!("OUT_DIR"), "/materials.rs"));
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct CliError {
    status: u8,
    message: String,
}
fn invalid(message: impl Into<String>) -> CliError {
    CliError {
        status: 2,
        message: message.into(),
    }
}
fn output(writer: &mut impl Write, text: &str) -> Result<(), CliError> {
    writer.write_all(text.as_bytes()).map_err(|error| CliError {
        status: 1,
        message: error.to_string(),
    })
}
fn overview(writer: &mut impl Write, operation: &str) -> Result<(), CliError> {
    if operation == "help" {
        output(
            writer,
            "sdev — 人とAIが仕様を起点に開発するためのフレームワーク\n\n読むガイドを作業名から選んでください。\n",
        )?;
    } else {
        output(
            writer,
            "内蔵の既定テンプレートの種類を選んでください。Markdown本文を表示するだけです。利用先の指示・規約で対象用と指定されたテンプレートがあれば、そちらを優先してください。\n",
        )?;
    }
    for material in MATERIALS
        .iter()
        .filter(|material| material.operation == operation)
    {
        output(
            writer,
            &format!(
                "  sdev {} {:<12} {}\n",
                material.operation, material.topic, material.title
            ),
        )?;
    }
    if operation == "help" {
        output(
            writer,
            "\n  sdev template              テンプレートの種類を確認\n  sdev --version             バージョン\n\n利用先のProduct → 機能のRequirements → Designを前提に進めます。\nsdev自身の仕様ではなく、作業するプロジェクトの文書を参照してください。\n",
        )?;
    }
    Ok(())
}
fn material(writer: &mut impl Write, operation: &str, topic: &str) -> Result<(), CliError> {
    let chosen = MATERIALS
        .iter()
        .find(|material| material.operation == operation && material.topic == topic)
        .ok_or_else(|| {
            invalid(format!(
                "不明な{}: {topic}\nsdev {}で選択肢を確認してください。",
                if operation == "help" {
                    "ガイド"
                } else {
                    "テンプレート"
                },
                if operation == "help" {
                    "help"
                } else {
                    "template"
                }
            ))
        })?;
    output(writer, chosen.body)
}
fn run(args: &[String], writer: &mut impl Write) -> Result<(), CliError> {
    let values: Vec<&str> = args.iter().map(String::as_str).collect();
    match values.as_slice() {
        [] | ["--help" | "-h" | "help"] => overview(writer, "help"),
        ["--version" | "-V"] => output(writer, &format!("sdev {VERSION}\n")),
        ["help", "--help" | "-h"] => overview(writer, "help"),
        ["help", topic] => material(writer, "help", topic),
        ["template"] | ["template", "--help" | "-h"] => overview(writer, "template"),
        ["template", topic] => material(writer, "template", topic),
        _ => Err(invalid(
            "コマンドまたは引数を確認してください。sdev --helpで使い方を確認できます。",
        )),
    }
}
fn main() -> ExitCode {
    let args = match env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| invalid("引数はUTF-8で指定してください。"))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(error.status);
        }
    };
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    match run(&args, &mut writer) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(error.status)
        }
    }
}
