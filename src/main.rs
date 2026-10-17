mod init;
#[cfg(unix)]
mod launcher;
mod lint;

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
            "\n  sdev init                  プロジェクトの参照指示と起動方法を準備\n  sdev lint                  文書のリンク切れと参照方向を検査\n  sdev template              テンプレートの種類を確認\n  sdev --version             バージョン\n\n利用先のProduct → 機能のRequirements → Designを前提に進めます。\nsdev自身の仕様ではなく、作業するプロジェクトの文書を参照してください。\n",
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
fn run(args: &[String], writer: &mut impl Write) -> Result<u8, CliError> {
    let values: Vec<&str> = args.iter().map(String::as_str).collect();
    if values.first() == Some(&"lint") {
        return lint_command(&values[1..], writer);
    }
    if values.first() == Some(&"init") {
        return init_command(&values[1..], writer);
    }
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
    .map(|()| 0)
}
const INIT_HELP: &str = "sdev init [--path PATH]\n\nプロジェクトのルートで実行し、AGENTS.mdへガイドとテンプレートを選ぶ指示を追加します。\nmacOS・Linuxでは./bin/sdevにコミット用の起動スクリプトを生成します。\n--path ./scripts/sdevのように配置先を変更でき、参照指示もそのパスを使います。\n初回の起動時に固定バージョンの本体を取得し、.gitignoreには/.sdev/を追加します。\nWindowsでは手動配置した./bin/sdev.exeを参照し、--pathは使いません。\n既存の本文とsdevの指示は保持します。別のスクリプトは上書きしません。\n仕様ファイルや空の仕様ディレクトリは作成しません。\n終了コード: 0 成功 / 1 ファイルや出力の処理失敗 / 2 引数の誤り\n";
fn init_command(args: &[&str], writer: &mut impl Write) -> Result<u8, CliError> {
    match args {
        ["--help" | "-h"] => output(writer, INIT_HELP)?,
        [] | ["--path", _] => {
            let path = args.get(1).copied().unwrap_or("./bin/sdev");
            #[cfg(unix)]
            let path = launcher::normalize_path(path).map_err(invalid)?;
            #[cfg(windows)]
            let path = if args.is_empty() {
                path.to_owned()
            } else {
                return Err(invalid("Windowsのinitでは--pathを使えません。"));
            };
            let body = env::current_dir()
                .map_err(|error| error.to_string())
                .and_then(|root| init::initialize(&root, &path))
                .map_err(|message| CliError { status: 1, message })?;
            output(writer, &body)?;
        }
        _ => {
            return Err(invalid(format!(
                "initの引数を確認してください。\n{INIT_HELP}"
            )));
        }
    }
    Ok(0)
}
const LINT_HELP: &str = "sdev lint [--format text|json]\n\n現在のディレクトリをルートとしてREADME.md・AGENTS.md・docs/**/*.mdを検査します。\n既定はtext、AI向けの構造化出力は--format jsonです。設定ファイルや外部URLへの通信は使いません。\n終了コード: 0 検査成功 / 1 規則違反 / 2 検査未完了・引数や出力の処理失敗\n規則: sdev help documents\n";
fn lint_command(args: &[&str], writer: &mut impl Write) -> Result<u8, CliError> {
    let format = match args {
        ["--help" | "-h"] => {
            writer
                .write_all(LINT_HELP.as_bytes())
                .map_err(|e| invalid(e.to_string()))?;
            return Ok(0);
        }
        [] | ["--format", "text"] => "text",
        ["--format", "json"] => "json",
        _ => {
            return Err(invalid(format!(
                "lintの引数を確認してください。\n{LINT_HELP}"
            )));
        }
    };
    let report = match env::current_dir() {
        Ok(root) => lint::check(&root),
        Err(error) => {
            let mut report = lint::Report::default();
            report.add(
                std::path::Path::new(""),
                None,
                "discovery-failed",
                format!("作業ディレクトリを取得できません: {error}"),
            );
            report.finish()
        }
    };
    let body = if format == "json" {
        report.json()
    } else {
        report.text()
    };
    writer
        .write_all(body.as_bytes())
        .map_err(|e| invalid(e.to_string()))?;
    Ok(report.code())
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
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(error.status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn lint_output_errors_use_two_and_existing_operations_keep_one() {
        for args in [vec!["lint", "--format", "json"], vec!["lint", "--help"]] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert_eq!(run(&args, &mut Broken).err().unwrap().status, 2);
        }
        assert_eq!(
            run(&["--version".into()], &mut Broken)
                .err()
                .unwrap()
                .status,
            1
        );
    }
}
