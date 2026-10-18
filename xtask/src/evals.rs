//! Local, session-driven evaluations of intake responses.
use crate::support::{self, Result, TemporaryDirectory};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod judge;
pub use judge::require_judgment;

#[path = "../../evals/help/consultation/EVAL.rs"]
mod consultation;
#[path = "../../evals/help/design/EVAL.rs"]
mod design;
#[path = "../../evals/help/implementation/EVAL.rs"]
mod implementation;
#[path = "../../evals/help/product/EVAL.rs"]
mod product;
#[path = "../../evals/help/requirements-change/EVAL.rs"]
mod requirements_change;
#[path = "../../evals/help/requirements-followup/EVAL.rs"]
mod requirements_followup;
#[path = "../../evals/help/version/EVAL.rs"]
mod version;

const GUIDES: [&str; 6] = [
    "product",
    "requirements",
    "design",
    "documents",
    "workflow",
    "scenarios",
];
const INSTRUCTIONS: &str = "Read `./sdev help` first. Choose and read the guides relevant to the user's request, then carry out the request. Respond in the user's language.\n";
const CONFIG: &str = "language = \"en\"\ntranslations = [\"ja\"]\n";
const RUNTIME: &str = if cfg!(windows) {
    ".sdev-runtime.exe"
} else {
    ".sdev-runtime"
};
const WRAPPER: &str = if cfg!(windows) { "sdev.exe" } else { "sdev" };
const USAGE: &str = "cargo xtask eval prepare --binary /absolute/sdev --output /absolute/new-run [--help-dir /absolute/baseline-dir] [--case name,name]\ncargo xtask eval judge-prepare --output /absolute/run\ncargo xtask eval score --output /absolute/run";

struct Case {
    name: &'static str,
    prompt: &'static str,
    evaluation: &'static str,
    rubric: &'static str,
    check: fn(&Run) -> Result<()>,
}

macro_rules! case {
    ($name:literal, $module:ident) => {
        Case {
            name: $name,
            prompt: include_str!(concat!("../../evals/help/", $name, "/PROMPT.md")),
            evaluation: include_str!(concat!("../../evals/help/", $name, "/EVAL.rs")),
            rubric: include_str!(concat!("../../evals/help/", $name, "/RUBRIC.json")),
            check: $module::check,
        }
    };
}

const CASES: [Case; 7] = [
    case!("consultation", consultation),
    case!("requirements-change", requirements_change),
    case!("requirements-followup", requirements_followup),
    case!("product", product),
    case!("design", design),
    case!("version", version),
    case!("implementation", implementation),
];

pub struct Run {
    pub name: String,
    pub directory: PathBuf,
    pub judge_directory: PathBuf,
    pub guides: BTreeSet<String>,
    pub trace: Vec<Value>,
    pub response: String,
    pub snapshots: Value,
}

pub fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("Missing string: {key}").into())
}

fn executable_copy(source: &Path, target: &Path) -> Result<()> {
    fs::copy(source, target)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

fn hashes(snapshots: &Value) -> Result<Value> {
    let mut values = Map::new();
    for (key, value) in snapshots.as_object().ok_or("Snapshots must be an object")? {
        values.insert(
            key.clone(),
            json!(support::digest(
                value.as_str().ok_or("Snapshot must be text")?.as_bytes()
            )),
        );
    }
    Ok(Value::Object(values))
}

pub fn run(args: &[&str]) -> Result<()> {
    let Some((command, arguments)) = args.split_first() else {
        return Err(USAGE.into());
    };
    let mut options = BTreeMap::new();
    for pair in arguments.chunks(2) {
        require(
            pair.len() == 2 && pair[0].starts_with("--") && !pair[1].starts_with("--"),
            USAGE,
        )?;
        require(
            options.insert(pair[0], pair[1]).is_none(),
            "Duplicate option",
        )?;
    }
    let output = Path::new(options.get("--output").ok_or(USAGE)?);
    require(output.is_absolute(), "--output must be absolute")?;
    match *command {
        "prepare" => {
            require(
                options
                    .keys()
                    .all(|key| ["--binary", "--output", "--help-dir", "--case"].contains(key)),
                USAGE,
            )?;
            let binary = Path::new(options.get("--binary").ok_or(USAGE)?);
            let baseline = options.get("--help-dir").map(Path::new);
            let selected: Vec<&str> = options
                .get("--case")
                .map(|names| names.split(',').collect())
                .unwrap_or_else(|| CASES.iter().map(|case| case.name).collect());
            prepare(binary, output, baseline, &selected)
        }
        "score" => {
            require(options.len() == 1, USAGE)?;
            score(output)
        }
        "judge-prepare" => {
            require(options.len() == 1, USAGE)?;
            judge::prepare(output)
        }
        _ => Err(USAGE.into()),
    }
}

fn prepare(binary: &Path, output: &Path, baseline: Option<&Path>, selected: &[&str]) -> Result<()> {
    require(binary.is_absolute(), "--binary must be absolute")?;
    require(
        baseline.is_none_or(Path::is_absolute),
        "--help-dir must be absolute",
    )?;
    require(
        !selected.is_empty() && selected.iter().collect::<BTreeSet<_>>().len() == selected.len(),
        "Select distinct cases",
    )?;
    for name in selected {
        require(CASES.iter().any(|case| case.name == *name), "Unknown case")?;
    }
    fs::create_dir_all(output.parent().ok_or("Output has no parent")?)?;
    fs::create_dir(output)?;
    let wrapper = env::current_exe()?;
    let mut snapshots = Map::new();
    let capture =
        |args: &[&str]| support::output(Command::new(binary).args(args).current_dir(output));
    snapshots.insert("version".into(), json!(capture(&["--version"])?));
    for language in ["en", "ja"] {
        let help = match baseline {
            Some(directory) => {
                fs::read_to_string(directory.join(format!("baseline.{language}.txt")))?
            }
            None => capture(&["help", "--lang", language])?,
        };
        snapshots.insert(format!("help.{language}"), json!(help));
        snapshots.insert(
            format!("template.{language}"),
            json!(capture(&["template", "--lang", language])?),
        );
        for guide in GUIDES {
            snapshots.insert(
                format!("help.{guide}.{language}"),
                json!(capture(&["help", guide, "--lang", language])?),
            );
        }
        for template in ["product", "requirements", "design"] {
            snapshots.insert(
                format!("template.{template}.{language}"),
                json!(capture(&["template", template, "--lang", language])?),
            );
        }
    }
    let snapshots = Value::Object(snapshots);
    let mut entries = Vec::new();
    for name in selected {
        let case = CASES.iter().find(|case| case.name == *name).unwrap();
        let directory = output.join(name);
        fs::create_dir(&directory)?;
        fs::write(directory.join("PROMPT.md"), case.prompt)?;
        fs::write(directory.join("AGENTS.md"), INSTRUCTIONS)?;
        fs::write(directory.join("sdev.toml"), CONFIG)?;
        fs::write(
            directory.join(".sdev-snapshots.json"),
            serde_json::to_vec(&snapshots)?,
        )?;
        executable_copy(&wrapper, &directory.join(WRAPPER))?;
        executable_copy(binary, &directory.join(RUNTIME))?;
        judge::criteria(case.rubric)?;
        entries.push(json!({"name": name, "promptHash": support::digest(case.prompt.as_bytes()), "evaluationHash": support::digest(case.evaluation.as_bytes()), "rubricHash": support::digest(case.rubric.as_bytes())}));
    }
    let manifest = json!({
        "schemaVersion": 3, "createdAtUnixMs": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        "runtime": {"os": env::consts::OS, "arch": env::consts::ARCH},
        "binaryHash": support::digest(&fs::read(binary)?), "runtimeBinary": RUNTIME,
        "wrapper": WRAPPER, "wrapperHash": support::digest(&fs::read(wrapper)?),
        "snapshotHashes": hashes(&snapshots)?, "cases": entries,
    });
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("Prepared {} cases in {}", selected.len(), output.display());
    for name in selected {
        println!("  {name}");
    }
    Ok(())
}

fn command_key(args: &[String]) -> Option<String> {
    let mut words = Vec::new();
    let mut language = "en";
    let mut index = 0;
    while index < args.len() {
        let word = args[index].as_str();
        if word == "--lang" {
            index += 1;
            language = args.get(index)?.as_str();
        } else if let Some(value) = word.strip_prefix("--lang=") {
            language = value;
        } else {
            words.push(word);
        }
        index += 1;
    }
    Some(match words.as_slice() {
        ["--version" | "-V"] => "version".into(),
        [] | ["help" | "--help" | "-h"] | ["help", "--help" | "-h"] => format!("help.{language}"),
        ["help", guide] => format!("help.{guide}.{language}"),
        ["template"] | ["template", "--help" | "-h"] => format!("template.{language}"),
        ["template", name] => format!("template.{name}.{language}"),
        ["lint", ..] => "lint".into(),
        _ => return None,
    })
}

fn observe(directory: &Path, args: &[String]) -> Result<(i32, String, String)> {
    let key = command_key(args);
    let snapshots: Value =
        serde_json::from_slice(&fs::read(directory.join(".sdev-snapshots.json"))?)?;
    let (exit_code, stdout, stderr) = if key.as_deref() == Some("lint") {
        match support::bounded_output(
            Command::new(directory.join(RUNTIME))
                .args(args)
                .current_dir(directory),
            Duration::from_secs(30),
        ) {
            Ok(output) => (
                output.status.code().unwrap_or(2),
                String::from_utf8_lossy(&output.stdout).into_owned(),
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ),
            Err(error) => (2, String::new(), format!("{error}\n")),
        }
    } else if let Some(value) = key.as_deref().and_then(|key| snapshots[key].as_str()) {
        (0, value.into(), String::new())
    } else {
        (
            2,
            String::new(),
            format!("Unsupported evaluation command: {args:?}\n"),
        )
    };
    let mut event = json!({"args": args, "key": key, "exitCode": exit_code, "stdoutHash": support::digest(stdout.as_bytes()), "atUnixMs": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()});
    if key.as_deref() == Some("lint") {
        event["stdout"] = json!(stdout);
        event["stderr"] = json!(stderr);
    }
    let mut trace = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("trace.jsonl"))?;
    let mut record = serde_json::to_vec(&event)?;
    record.push(b'\n');
    trace.write_all(&record)?;
    Ok((exit_code, stdout, stderr))
}

pub fn observe_wrapper(args: &[String]) -> Result<i32> {
    let directory = env::current_exe()?
        .parent()
        .ok_or("Wrapper has no parent")?
        .to_owned();
    let (code, stdout, stderr) = observe(&directory, args)?;
    io::stdout().write_all(stdout.as_bytes())?;
    io::stderr().write_all(stderr.as_bytes())?;
    Ok(code)
}

fn read_run(output: &Path, manifest: &Value, entry: &Value) -> Result<Run> {
    let name = text(entry, "name")?;
    let case = CASES
        .iter()
        .find(|case| case.name == name)
        .ok_or("Unknown case in manifest")?;
    let directory = output.join(name);
    for (key, actual) in [
        ("promptHash", support::digest(case.prompt.as_bytes())),
        (
            "evaluationHash",
            support::digest(case.evaluation.as_bytes()),
        ),
        ("rubricHash", support::digest(case.rubric.as_bytes())),
    ] {
        require(
            text(entry, key)? == actual,
            "Fixture changed since prepare; prepare a fresh run",
        )?;
    }
    require(
        fs::read(directory.join("PROMPT.md"))? == case.prompt.as_bytes(),
        "Subject prompt changed",
    )?;
    require(
        fs::read_to_string(directory.join("AGENTS.md"))? == INSTRUCTIONS,
        "Instructions changed",
    )?;
    require(
        fs::read_to_string(directory.join("sdev.toml"))? == CONFIG,
        "Language settings changed",
    )?;
    require(
        text(manifest, "wrapper")? == WRAPPER && text(manifest, "runtimeBinary")? == RUNTIME,
        "Invalid executable names",
    )?;
    for (filename, key) in [(WRAPPER, "wrapperHash"), (RUNTIME, "binaryHash")] {
        require(
            support::digest(&fs::read(directory.join(filename))?) == text(manifest, key)?,
            "Prepared executable changed",
        )?;
    }
    let snapshots: Value =
        serde_json::from_slice(&fs::read(directory.join(".sdev-snapshots.json"))?)?;
    require(
        hashes(&snapshots)? == manifest["snapshotHashes"],
        "Snapshots changed",
    )?;
    let response = fs::read_to_string(directory.join("response.md"))?;
    require(!response.trim().is_empty(), "Final response is empty")?;
    let trace = fs::read_to_string(directory.join("trace.jsonl"))?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<Vec<Value>, _>>()?;
    validate_trace(&trace, manifest)?;
    let guides = trace
        .iter()
        .filter_map(|event| {
            let parts: Vec<&str> = event["key"].as_str()?.split('.').collect();
            (parts.len() == 3
                && parts[0] == "help"
                && GUIDES.contains(&parts[1])
                && ["en", "ja"].contains(&parts[2]))
            .then(|| parts[1].to_owned())
        })
        .collect();
    Ok(Run {
        name: name.into(),
        directory,
        judge_directory: output.join("judges").join(name),
        guides,
        trace,
        response,
        snapshots,
    })
}

fn validate_trace(trace: &[Value], manifest: &Value) -> Result<()> {
    require(!trace.is_empty(), "No observed commands")?;
    let mut last_lint = None;
    for event in trace {
        let args = event["args"]
            .as_array()
            .ok_or("Missing command arguments")?
            .iter()
            .map(|arg| {
                arg.as_str()
                    .map(str::to_owned)
                    .ok_or("Invalid command argument")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let key = command_key(&args).ok_or("Unsupported observed command")?;
        require(
            event["key"].as_str() == Some(&key),
            "Command key does not match arguments",
        )?;
        let status = event["exitCode"].as_i64().ok_or("Missing exit status")?;
        if key == "lint" {
            require(
                text(event, "stdoutHash")? == support::digest(text(event, "stdout")?.as_bytes()),
                "Unexpected lint output",
            )?;
            if args
                .iter()
                .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
            {
                require(status == 0, "Lint help command failed")?;
            } else {
                last_lint = Some(status);
            }
        } else {
            require(status == 0, "An observed command failed")?;
            require(
                manifest["snapshotHashes"][&key]
                    .as_str()
                    .is_some_and(|expected| event["stdoutHash"].as_str() == Some(expected)),
                "Unexpected command output",
            )?;
        }
    }
    require(
        last_lint.is_none_or(|code| code == 0),
        "Last lint command failed",
    )
}

fn read_manifest(output: &Path) -> Result<Value> {
    let manifest: Value = serde_json::from_slice(&fs::read(output.join("manifest.json"))?)?;
    require(
        manifest["schemaVersion"] == 3,
        "Unsupported manifest; prepare a fresh Rust evaluation",
    )?;
    let cases = manifest["cases"].as_array().ok_or("Missing cases")?;
    require(!cases.is_empty(), "Manifest has no cases")?;
    let mut seen = BTreeSet::new();
    for entry in cases {
        let name = text(entry, "name")?;
        require(seen.insert(name), "Duplicate case in manifest")?;
        require(
            CASES.iter().any(|case| case.name == name),
            "Unknown case in manifest",
        )?;
    }
    Ok(manifest)
}

fn score(output: &Path) -> Result<()> {
    let manifest = read_manifest(output)?;
    let cases = manifest["cases"].as_array().unwrap();
    let mut failures = 0;
    let mut errors = 0;
    for entry in cases {
        let name = text(entry, "name")?;
        let run = match read_run(output, &manifest, entry) {
            Ok(run) => run,
            Err(error) => {
                errors += 1;
                println!("ERROR {name}: {error}");
                continue;
            }
        };
        let guides = run.guides.iter().cloned().collect::<Vec<_>>().join(", ");
        println!("OBSERVED {name}: guides=[{guides}]");
        match (CASES.iter().find(|case| case.name == name).unwrap().check)(&run) {
            Ok(()) => println!("PASS {name}"),
            Err(error) if error.is::<judge::InvalidJudgment>() => {
                errors += 1;
                println!("ERROR {name}: {error}");
            }
            Err(error) => {
                failures += 1;
                println!("FAIL {name}: {error}");
            }
        }
    }
    println!(
        "{} passed; {failures} failed; {errors} errors",
        cases.len() - failures - errors
    );
    require(failures == 0 && errors == 0, "Evaluation failed")
}

pub fn check_sum(directory: &Path) -> Result<()> {
    require(directory.join("sum.rs").is_file(), "Implement sum.rs")?;
    let temporary = TemporaryDirectory::new("eval-sum")?;
    let source = format!(
        "#[path = {:?}] mod subject;\nfn main() {{\n assert_eq!(subject::sum(2, 3), Some(5));\n assert_eq!(subject::sum(-3, 3), Some(0));\n assert_eq!(subject::sum(i64::MAX, 0), Some(i64::MAX));\n assert_eq!(subject::sum(i64::MIN, 0), Some(i64::MIN));\n assert_eq!(subject::sum(i64::MAX, 1), None);\n assert_eq!(subject::sum(i64::MIN, -1), None);\n}}\n",
        directory.join("sum.rs")
    );
    let probe = temporary.0.join("probe.rs");
    let executable = temporary
        .0
        .join(if cfg!(windows) { "probe.exe" } else { "probe" });
    fs::write(&probe, source)?;
    let toolchain = support::Toolchain::new()?;
    for mut command in [
        {
            let mut command = Command::new("rustup");
            command
                .args(["run", &toolchain.channel, "rustc", "--edition=2024"])
                .arg(&probe)
                .arg("-o")
                .arg(&executable);
            command
        },
        Command::new(&executable),
    ] {
        let output = support::bounded_output(&mut command, Duration::from_secs(10))?;
        require(
            output.status.success(),
            &format!(
                "sum.rs verification failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) struct Fixture {
        _temporary: TemporaryDirectory,
        pub(super) output: PathBuf,
        pub(super) directory: PathBuf,
    }

    fn fixture(name: &str) -> Fixture {
        let temporary = TemporaryDirectory::new("eval-check").unwrap();
        let output = temporary.0.clone();
        let directory = output.join(name);
        fs::create_dir(&directory).unwrap();
        let case = CASES.iter().find(|case| case.name == name).unwrap();
        let snapshots = json!({"help.en": "Choose a guide\n", "help.ja": "ガイドを選ぶ\n", "version": "sdev 0.4.2\n", "template.en": "Templates\n", "help.workflow.en": "Dialogue and scope\n"});
        for (path, bytes) in [
            ("PROMPT.md", case.prompt.as_bytes()),
            ("AGENTS.md", INSTRUCTIONS.as_bytes()),
            ("sdev.toml", CONFIG.as_bytes()),
            (WRAPPER, b"wrapper".as_slice()),
            (RUNTIME, b"runtime".as_slice()),
            ("response.md", b"sdev 0.4.2\n".as_slice()),
        ] {
            fs::write(directory.join(path), bytes).unwrap();
        }
        fs::write(
            directory.join(".sdev-snapshots.json"),
            serde_json::to_vec(&snapshots).unwrap(),
        )
        .unwrap();
        let manifest = json!({
            "schemaVersion": 3, "wrapper": WRAPPER, "wrapperHash": support::digest(b"wrapper"),
            "runtimeBinary": RUNTIME, "binaryHash": support::digest(b"runtime"),
            "snapshotHashes": hashes(&snapshots).unwrap(),
            "cases": [{"name": name, "promptHash": support::digest(case.prompt.as_bytes()), "evaluationHash": support::digest(case.evaluation.as_bytes()), "rubricHash": support::digest(case.rubric.as_bytes())}],
        });
        fs::write(
            output.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Fixture {
            _temporary: temporary,
            output,
            directory,
        }
    }

    fn invoke(fixture: &Fixture, args: &[&str]) -> i32 {
        observe(
            &fixture.directory,
            &args
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .0
    }

    pub(super) fn observed_version() -> Fixture {
        let fixture = fixture("version");
        assert_eq!(invoke(&fixture, &["help"]), 0);
        assert_eq!(invoke(&fixture, &["-V"]), 0);
        fixture
    }

    pub(super) fn read_fixture(fixture: &Fixture) -> Run {
        let manifest = read_manifest(&fixture.output).unwrap();
        read_run(&fixture.output, &manifest, &manifest["cases"][0]).unwrap()
    }

    #[test]
    fn version_passes_only_with_observation_and_correct_answer() {
        let fixture = observed_version();
        judge::tests::all_pass(&fixture.output);
        score(&fixture.output).unwrap();
        for answer in ["", "sdev 0.4.3", "0.4.2 or 0.4.3", "I read the version"] {
            fs::write(fixture.directory.join("response.md"), answer).unwrap();
            assert!(score(&fixture.output).is_err(), "{answer}");
        }
        fs::write(fixture.directory.join("response.md"), "0.4.2").unwrap();
        fs::remove_file(fixture.directory.join("trace.jsonl")).unwrap();
        assert!(score(&fixture.output).is_err());
    }

    #[test]
    fn guide_selection_is_observation_and_does_not_determine_the_score() {
        let fixture = fixture("version");
        assert_eq!(invoke(&fixture, &["-V"]), 0);
        assert_eq!(invoke(&fixture, &["help", "workflow"]), 0);
        judge::tests::all_pass(&fixture.output);
        score(&fixture.output).unwrap();
    }

    #[test]
    fn empty_or_incomplete_cases_fail_instead_of_false_passing() {
        let fixture = fixture("version");
        fs::write(fixture.directory.join("trace.jsonl"), "").unwrap();
        assert!(score(&fixture.output).is_err());
        fs::remove_file(fixture.directory.join("response.md")).unwrap();
        assert!(score(&fixture.output).is_err());
        fs::remove_dir_all(&fixture.directory).unwrap();
        assert!(score(&fixture.output).is_err());
        fs::write(
            fixture.output.join("manifest.json"),
            r#"{"schemaVersion":3,"cases":[]}"#,
        )
        .unwrap();
        assert!(score(&fixture.output).is_err());
    }

    #[test]
    fn prepared_inputs_and_evaluators_cannot_change() {
        for path in [
            "PROMPT.md",
            "AGENTS.md",
            "sdev.toml",
            WRAPPER,
            RUNTIME,
            ".sdev-snapshots.json",
        ] {
            let fixture = observed_version();
            fs::write(fixture.directory.join(path), "changed").unwrap();
            assert!(score(&fixture.output).is_err(), "{path}");
        }
        for key in ["evaluationHash", "rubricHash"] {
            let fixture = observed_version();
            let path = fixture.output.join("manifest.json");
            let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            manifest["cases"][0][key] = json!("wrong evaluator");
            fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
            assert!(score(&fixture.output).is_err());
        }
    }

    #[test]
    fn aliases_select_the_same_output_and_unknown_commands_fail() {
        for (args, expected) in [
            (vec!["-V"], "version"),
            (vec!["help", "--help"], "help.en"),
            (vec!["template", "--help"], "template.en"),
            (vec!["--lang", "ja", "help"], "help.ja"),
            (vec!["help", "workflow", "--lang=en"], "help.workflow.en"),
        ] {
            let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
            assert_eq!(command_key(&args).as_deref(), Some(expected));
        }
        let fixture = observed_version();
        assert_eq!(invoke(&fixture, &["help", "--help"]), 0);
        assert_eq!(invoke(&fixture, &["template", "--help"]), 0);
        judge::tests::all_pass(&fixture.output);
        score(&fixture.output).unwrap();
        assert_eq!(invoke(&fixture, &["unknown"]), 2);
        assert!(score(&fixture.output).is_err());
    }

    #[test]
    fn concurrent_commands_keep_trace_records_separate() {
        let fixture = fixture("version");
        assert_eq!(invoke(&fixture, &["help"]), 0);
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let fixture = &fixture;
                scope.spawn(move || {
                    for _ in 0..10 {
                        assert_eq!(invoke(fixture, &["-V"]), 0);
                    }
                });
            }
        });
        let trace = fs::read_to_string(fixture.directory.join("trace.jsonl")).unwrap();
        assert_eq!(trace.lines().count(), 161);
        judge::tests::all_pass(&fixture.output);
        score(&fixture.output).unwrap();
    }

    #[test]
    fn live_lint_can_recover_after_fixing_documents() {
        let fixture = observed_version();
        let source = fixture.output.join("runtime.rs");
        fs::write(&source, "fn main() { println!(\"checked documents\"); if std::env::args().any(|arg| arg == \"--help\" || arg == \"-h\") { return; } if std::path::Path::new(\"lint-fails\").exists() { std::process::exit(1); } }").unwrap();
        let toolchain = support::Toolchain::new().unwrap();
        support::run(
            Command::new("rustup")
                .args(["run", &toolchain.channel, "rustc"])
                .arg(&source)
                .arg("-o")
                .arg(fixture.directory.join(RUNTIME)),
        )
        .unwrap();
        let path = fixture.output.join("manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["binaryHash"] = json!(support::digest(
            &fs::read(fixture.directory.join(RUNTIME)).unwrap()
        ));
        fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        fs::write(fixture.directory.join("lint-fails"), "").unwrap();
        assert_eq!(invoke(&fixture, &["lint"]), 1);
        assert!(score(&fixture.output).is_err());
        assert_eq!(invoke(&fixture, &["lint", "--help"]), 0);
        assert!(score(&fixture.output).is_err());
        assert_eq!(invoke(&fixture, &["lint", "-h"]), 0);
        assert!(score(&fixture.output).is_err());
        fs::remove_file(fixture.directory.join("lint-fails")).unwrap();
        assert_eq!(invoke(&fixture, &["lint"]), 0);
        judge::tests::all_pass(&fixture.output);
        score(&fixture.output).unwrap();
        let trace_path = fixture.directory.join("trace.jsonl");
        let trace = fs::read_to_string(&trace_path)
            .unwrap()
            .replace("checked documents", "forged documents");
        fs::write(trace_path, trace).unwrap();
        assert!(score(&fixture.output).is_err());
    }

    #[test]
    fn rust_implementation_checks_arithmetic_boundaries() {
        let temporary = TemporaryDirectory::new("eval-sum-check").unwrap();
        fs::write(
            temporary.0.join("sum.rs"),
            "pub fn sum(a: i64, b: i64) -> Option<i64> { a.checked_add(b) }",
        )
        .unwrap();
        check_sum(&temporary.0).unwrap();
        fs::write(
            temporary.0.join("sum.rs"),
            "pub fn sum(a: i64, b: i64) -> Option<i64> { Some(a + b) }",
        )
        .unwrap();
        assert!(check_sum(&temporary.0).is_err());
    }
}
