mod init;
mod language;
#[cfg(unix)]
mod launcher;
mod lint;
mod review;
mod update;

use language::{Language, ProjectConfig};
use std::{
    env,
    io::{self, Write},
    process::ExitCode,
};

struct Material {
    operation: &'static str,
    topic: &'static str,
    language: &'static str,
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
fn overview(writer: &mut impl Write, operation: &str, language: Language) -> Result<(), CliError> {
    output(
        writer,
        if operation == "help" {
            language.select("sdev — specifications-first development with people and AI\n\nChoose a guide for your task.\n", "sdev — 人とAIが仕様を起点に開発するためのフレームワーク\n\n読むガイドを作業名から選んでください。\n")
        } else {
            language.select("Choose a built-in fallback template. Commands display Markdown only. Prefer any template designated for the document by the consuming project.\n", "内蔵の既定テンプレートの種類を選んでください。Markdown本文を表示するだけです。利用先の指示・規約で対象用と指定されたテンプレートがあれば、そちらを優先してください。\n")
        },
    )?;
    for material in MATERIALS
        .iter()
        .filter(|material| material.operation == operation && material.language == language.code())
    {
        output(
            writer,
            &format!(
                "  sdev {} {:<12} --lang {}  {}\n",
                material.operation,
                material.topic,
                language.code(),
                material.title
            ),
        )?;
    }
    if operation == "help" {
        output(writer, language.select(
            "\n  sdev review                Review specifications and Git differences in one HTML file\n",
            "\n  sdev review                仕様とGit差分を単一HTMLでレビュー\n",
        ))?;
        output(writer, language.select(
            "\n  sdev init                  Set up project instructions, languages, and a launcher\n  sdev update                Update the project launcher\n  sdev lint                  Check document links and reference directions\n  sdev template              List templates\n  sdev --version             Show version\n\nUse the consuming project's Product → Requirements → Design, not sdev's own specifications.\nSelect a language with --lang en|ja. Configure project defaults in sdev.toml.\n",
            "\n  sdev init                  プロジェクトの参照指示・言語・起動方法を準備\n  sdev update                プロジェクトの起動スクリプトを更新\n  sdev lint                  文書のリンク切れと参照方向を検査\n  sdev template              テンプレートの種類を確認\n  sdev --version             バージョン\n\n利用先のProduct → 機能のRequirements → Designを前提に進めます。\nsdev自身の仕様ではなく、作業するプロジェクトの文書を参照してください。\n--lang en|jaで言語を選択できます。プロジェクトの既定はsdev.tomlに設定します。\n"))?;
    }
    Ok(())
}
fn material(
    writer: &mut impl Write,
    operation: &str,
    topic: &str,
    language: Language,
) -> Result<(), CliError> {
    let available = |lang| {
        MATERIALS
            .iter()
            .find(|m| m.operation == operation && m.topic == topic && m.language == lang)
    };
    let chosen = if let Some(chosen) = available(language.code()) {
        chosen
    } else if let Some(chosen) = available("en") {
        eprintln!(
            "No {} translation for {operation} {topic}; using English.",
            language.code()
        );
        chosen
    } else {
        return Err(invalid(language.select(
            &format!("Unknown {operation} topic: {topic}\nUse sdev {operation} to list choices."),
            &format!(
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
            ),
        )));
    };
    output(writer, chosen.body)
}
fn options(args: &[String]) -> Result<(Vec<&str>, Option<Language>), CliError> {
    let mut values = Vec::new();
    let mut language = None;
    let mut args = args.iter();
    while let Some(value) = args.next() {
        let code = if value == "--lang" {
            Some(
                args.next()
                    .ok_or_else(|| invalid("--lang requires en or ja."))?
                    .as_str(),
            )
        } else {
            value.strip_prefix("--lang=")
        };
        if let Some(code) = code {
            if language.is_some() {
                return Err(invalid("Specify --lang only once."));
            }
            language = Some(Language::parse(code).map_err(invalid)?);
        } else {
            values.push(value.as_str());
        }
    }
    Ok((values, language))
}
fn run(args: &[String], writer: &mut impl Write) -> Result<u8, CliError> {
    let (values, explicit) = options(args)?;
    if matches!(values.as_slice(), ["--version" | "-V"]) {
        output(writer, &format!("sdev {VERSION}\n"))?;
        return Ok(0);
    }
    let root = env::current_dir().map_err(|e| invalid(e.to_string()))?;
    let config = ProjectConfig::read(&root).map_err(invalid)?;
    let language = explicit.unwrap_or_else(|| config.as_ref().map_or(Language::En, |c| c.language));
    let result = match values.as_slice() {
        ["lint", rest @ ..] => lint_command(rest, writer, language),
        ["review", rest @ ..] => {
            if matches!(rest, ["--help" | "-h"]) {
                output(writer, review::help(language)).map(|()| 0)
            } else {
                let arguments = review::arguments(rest).map_err(invalid)?;
                review::run(&arguments, &root, writer, language)
                    .map(|()| 0)
                    .map_err(|message| CliError { status: 1, message })
            }
        }
        ["init", rest @ ..] => init_command(rest, writer, language, explicit, config.as_ref()),
        ["update", rest @ ..] => update_command(rest, writer, language),
        [] | ["--help" | "-h" | "help"] | ["help", "--help" | "-h"] => {
            overview(writer, "help", language).map(|()| 0)
        }
        ["help", topic] => material(writer, "help", topic, language).map(|()| 0),
        ["template"] | ["template", "--help" | "-h"] => {
            overview(writer, "template", language).map(|()| 0)
        }
        ["template", topic] => material(writer, "template", topic, language).map(|()| 0),
        _ => Err(invalid(language.select(
            "Unknown command or arguments. See sdev --help.",
            "コマンドまたは引数を確認してください。sdev --helpで使い方を確認できます。",
        ))),
    };
    result.map_err(|mut error| {
        error.message = language.message(&error.message);
        error
    })
}
fn update_command(
    args: &[&str],
    writer: &mut impl Write,
    language: Language,
) -> Result<u8, CliError> {
    if args == ["--help"] || args == ["-h"] {
        output(writer, language.select(
            "sdev update [--version X.Y.Z] [--path PATH] [--verify] [--source-commit SHA] [--trusted-root FILE]\n\nRun at the project root. Update its generated macOS/Linux launcher to the latest stable or specified release.\nThe launcher supplies its own path; direct invocation defaults to ./bin/sdev. Use --path for another project-relative location.\nPreserve AGENTS.md, .gitignore, sdev.toml, specifications, and edited launchers. Review and commit the updated launcher.\nChecksums are always checked. --verify or SDEV_VERIFY=1 also verifies provenance; verification failures never fall back.\n--source-commit and --trusted-root require verification. Other installation methods must use their own updater.\nExit codes: 0 success / 1 update or output failure / 2 invalid arguments or configuration\n",
            "sdev update [--version X.Y.Z] [--path PATH] [--verify] [--source-commit SHA] [--trusted-root FILE]\n\nプロジェクトのルートで実行し、macOS・Linuxの生成済み起動スクリプトを最新安定版または指定版へ更新します。\n起動スクリプトの配置先を引き継ぎます。本体の直接実行では./bin/sdevを既定とし、別の相対配置先には--pathを使います。\nAGENTS.md・.gitignore・sdev.toml・仕様・編集済みスクリプトは保持します。更新したスクリプトは差分を確認してコミットしてください。\nチェックサムを毎回確認します。--verifyまたはSDEV_VERIFY=1では証明も検証し、失敗時に検証を弱めません。\n--source-commitと--trusted-rootには証明検証が必要です。ほかの導入方法では、その方法で更新してください。\n終了コード: 0 成功 / 1 更新・出力失敗 / 2 引数・設定の誤り\n"))?;
        return Ok(0);
    }
    let arguments = update::arguments(args).map_err(invalid)?;
    let report =
        update::run(&arguments, language).map_err(|message| CliError { status: 1, message })?;
    output(writer, &report).map(|()| 0)
}
fn init_command(
    args: &[&str],
    writer: &mut impl Write,
    language: Language,
    explicit: Option<Language>,
    existing: Option<&ProjectConfig>,
) -> Result<u8, CliError> {
    let help = language.select(
        "sdev init [--path PATH] [--lang en|ja] [--translations ja|en]\n\nRun at the project root. Create sdev.toml and append guide-selection instructions to AGENTS.md.\nOn macOS/Linux, create a committed launcher at ./bin/sdev; --path ./scripts/sdev chooses another location.\nInitialization requires curl and network access to pin SHA-256 values for all five release targets in the launcher. The first launch checks the binary against its committed hash and caches it in .sdev/. Add /.sdev/ to .gitignore.\nOn Windows, use the manually installed ./bin/sdev.exe; --path is unavailable.\nPreserve existing files and edited instructions. No specification files or empty directories are created.\nExit codes: 0 success / 1 file or output failure / 2 invalid arguments or configuration\n",
        "sdev init [--path PATH] [--lang en|ja] [--translations ja|en]\n\nプロジェクトのルートで実行し、sdev.tomlとAGENTS.mdの参照指示を準備します。\nmacOS・Linuxでは./bin/sdevにコミット用の起動スクリプトを生成します。\n--path ./scripts/sdevのように配置先を変更でき、参照指示もそのパスを使います。\n初期設定にはcurlと通信が必要です。全5対象のSHA-256を取得し、起動スクリプトへ固定します。\n初回起動ではコミット済みのSHAと本体を照合し、.sdev/へ保存します。.gitignoreには/.sdev/を追加します。\nWindowsでは手動配置した./bin/sdev.exeを参照し、--pathは使いません。\n既存の本文とsdevの指示は保持します。別のスクリプトは上書きしません。\n仕様ファイルや空の仕様ディレクトリは作成しません。\n終了コード: 0 成功 / 1 ファイルや出力の処理失敗 / 2 引数や設定の誤り\n");
    if matches!(args, ["--help" | "-h"]) {
        output(writer, help)?;
        return Ok(0);
    }
    let mut path = None;
    let mut translations = None;
    let mut args = args.iter();
    while let Some(&flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| invalid(format!("Missing value for {flag}.\n{help}")))?;
        match flag {
            "--path" if path.is_none() => path = Some(*value),
            "--translations" if translations.is_none() => {
                translations = Some(if value.is_empty() {
                    Vec::new()
                } else {
                    value
                        .split(',')
                        .map(Language::parse)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(invalid)?
                });
            }
            _ => return Err(invalid(format!("Invalid init arguments.\n{help}"))),
        }
    }
    #[cfg(windows)]
    if path.is_some() {
        return Err(invalid("--path is only available on macOS and Linux."));
    }
    let selected = ProjectConfig {
        language,
        translations: translations
            .clone()
            .unwrap_or_else(|| existing.map_or_else(Vec::new, |c| c.translations.clone())),
    };
    if let Some(existing) = existing
        && ((explicit.is_some() && existing.language != language)
            || (translations.is_some() && existing.translations != selected.translations))
    {
        return Err(invalid(
            "Language settings differ. Edit sdev.toml explicitly before reinitializing; no files were changed.",
        ));
    }
    selected.validate().map_err(invalid)?;
    let path = path.unwrap_or("bin/sdev");
    #[cfg(unix)]
    let path = launcher::normalize_path(path).map_err(invalid)?;
    #[cfg(windows)]
    let path = path.to_owned();
    let root = env::current_dir().map_err(|e| CliError {
        status: 1,
        message: e.to_string(),
    })?;
    let result = init::initialize(&root, &path, &selected)
        .map_err(|message| CliError { status: 1, message })?;
    output(writer, &language.message(&result))?;
    Ok(0)
}
fn lint_command(
    args: &[&str],
    writer: &mut impl Write,
    language: Language,
) -> Result<u8, CliError> {
    let help = language.select(
        "sdev lint [--format text|json] [--lang en|ja]\n\nCheck README.md, AGENTS.md, their language versions, and docs/**/*.md from the current directory.\nDefault output is text; --format json provides structured output for tools. No external URLs are fetched.\nExit codes: 0 pass / 1 violations / 2 incomplete, invalid arguments, or output failure\nRules: sdev help documents\n",
        "sdev lint [--format text|json] [--lang en|ja]\n\n現在のディレクトリをルートとしてREADME.md・AGENTS.md・各言語版・docs/**/*.mdを検査します。\n既定はtext、AI向けの構造化出力は--format jsonです。外部URLへの通信は使いません。\n終了コード: 0 検査成功 / 1 規則違反 / 2 検査未完了・引数や出力の処理失敗\n規則: sdev help documents --lang ja\n");
    let format = match args {
        ["--help" | "-h"] => {
            writer
                .write_all(help.as_bytes())
                .map_err(|e| invalid(e.to_string()))?;
            return Ok(0);
        }
        [] | ["--format", "text"] => "text",
        ["--format", "json"] => "json",
        _ => {
            let reason = language.select("Invalid lint arguments.", "lintの引数が不正です。");
            return Err(invalid(format!("{reason}\n{help}")));
        }
    };
    let root = env::current_dir().map_err(|e| invalid(e.to_string()))?;
    let report = lint::check(&root).with_language(language);
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
                .map_err(|_| invalid("Arguments must be UTF-8."))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(error.status);
        }
    };
    match run(&args, &mut io::stdout().lock()) {
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
