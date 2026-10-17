use crate::support::{self, Result, TemporaryDirectory, root};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    process::{Command, Stdio},
    time::Instant,
};

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

fn describe(values: &[f64]) -> Value {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    json!({"n":values.len(),"median_ms":median(values),"p10_ms":ordered[5],"p90_ms":ordered[53],"batch_medians_ms":values.chunks(20).map(median).collect::<Vec<_>>()})
}

fn order(seed: &mut u64) -> [usize; 2] {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    if *seed & 1 == 0 { [0, 1] } else { [1, 0] }
}

pub fn measure(with_rust: bool) -> Result<()> {
    let base = root().join("experiments/command-distribution");
    let names = ["bun", if with_rust { "rust" } else { "scriptc-dynamic" }];
    let binaries = [
        base.join("dist/bun-host"),
        base.join(if with_rust {
            "dist/rust-host"
        } else {
            "dist/scriptc-host"
        }),
    ];
    let directory = TemporaryDirectory::new("measurement")?;
    let inputs = ["docs/README.md", "docs/products/sdev/product.md"];
    let paths: Vec<String> = inputs
        .iter()
        .map(|file| root().join(file).to_string_lossy().into_owned())
        .collect();
    let cases = [
        ("version", vec!["--version"]),
        ("help", vec!["--help"]),
        ("docs", vec!["docs", "docs/templates/product.md"]),
        (
            "markdown",
            vec!["check", paths[0].as_str(), paths[1].as_str()],
        ),
    ];
    let invoke = |index: usize, args: &[&str], capture: bool| -> Result<Vec<u8>> {
        let mut command = Command::new(&binaries[index]);
        command
            .args(args)
            .current_dir(&directory.0)
            .env_clear()
            .env("PATH", "/usr/bin:/bin");
        if let Some(home) = env::var_os("HOME") {
            command.env("HOME", home);
        }
        if !capture {
            command.stdout(Stdio::null());
        }
        let result = command.output()?;
        if !result.status.success() {
            return Err(format!(
                "Measurement failed: {} {args:?}: {}",
                names[index],
                String::from_utf8_lossy(&result.stderr)
            )
            .into());
        }
        Ok(result.stdout)
    };
    let mut seed = 17_u64;
    let mut samples: BTreeMap<&str, BTreeMap<&str, Vec<f64>>> =
        names.iter().map(|name| (*name, BTreeMap::new())).collect();
    for (case, args) in &cases {
        for index in 0..binaries.len() {
            for _ in 0..5 {
                invoke(index, args, false)?;
            }
        }
        for _ in 0..60 {
            for index in order(&mut seed) {
                let start = Instant::now();
                invoke(index, args, false)?;
                samples
                    .get_mut(names[index])
                    .unwrap()
                    .entry(case)
                    .or_default()
                    .push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
    }
    let summary = |samples: &BTreeMap<&str, BTreeMap<&str, Vec<f64>>>| -> Value {
        json!(
            samples
                .iter()
                .map(|(name, cases)| (
                    *name,
                    cases
                        .iter()
                        .map(|(case, values)| (*case, describe(values)))
                        .collect::<BTreeMap<_, _>>()
                ))
                .collect::<BTreeMap<_, _>>()
        )
    };
    let mut result = json!({
        "measurement_runtime":"Rust std::time::Instant",
        "host":env::consts::OS,"machine":env::consts::ARCH,
        "method":"5 warmups, 60 alternating randomized fresh processes per binary/case; wall clock includes spawn and output, warm file cache; 3 batches of 20; no cold cache claim; xorshift64 seed 17 (different ordering from historical Python random)",
        "inputs":inputs.iter().map(|file| { let bytes=fs::read(root().join(file))?; Ok(json!({"path":file,"bytes":bytes.len(),"sha256":support::digest(&bytes)})) }).collect::<Result<Vec<_>>>()?,
        "summary":summary(&samples),"samples_ms":samples,
        "bytes":names.iter().zip(&binaries).map(|(name,path)| Ok((*name, fs::metadata(path)?.len()))).collect::<Result<BTreeMap<_,_>>>()?,
        "binary_sha256":names.iter().zip(&binaries).map(|(name,path)| Ok((*name, support::digest(&fs::read(path)?)))).collect::<Result<BTreeMap<_,_>>>()?
    });
    if with_rust {
        let stages = ["read_ms", "parse_ms", "references_ms", "total_ms"];
        let args = ["profile", paths[0].as_str(), paths[1].as_str()];
        let mut internal: BTreeMap<&str, BTreeMap<&str, Vec<f64>>> =
            names.iter().map(|name| (*name, BTreeMap::new())).collect();
        for index in 0..binaries.len() {
            for _ in 0..5 {
                invoke(index, &args, true)?;
            }
        }
        let mut counts = None;
        for _ in 0..60 {
            for index in order(&mut seed) {
                let value: Value = serde_json::from_slice(&invoke(index, &args, true)?)?;
                let current = json!({"files":value["files"],"links":value["links"],"missing":value["missing"]});
                if counts.as_ref().is_some_and(|counts| counts != &current) {
                    return Err("Profile counts differ".into());
                }
                counts = Some(current);
                for stage in stages {
                    internal
                        .get_mut(names[index])
                        .unwrap()
                        .entry(stage)
                        .or_default()
                        .push(value[stage].as_f64().ok_or("Missing profile stage")?);
                }
            }
        }
        result["internal"] = json!({"method":"Separate profile command, 5 warmups and 60 alternating fresh processes; stage clocks exclude startup, library initialization, dispatch, and JSON serialization/stdout; no clocks in normal check","counts":counts,"summary":summary(&internal),"samples_ms":internal});
    }
    let file = base.join(if with_rust {
        "results/startup-rust-xtask.json"
    } else {
        "results/startup-xtask.json"
    });
    fs::create_dir_all(file.parent().unwrap())?;
    fs::write(
        &file,
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    println!(
        "Saved new measurements to {}; historical results are unchanged",
        file.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_same_batch_and_percentile_positions_as_historical_measurement() {
        let values: Vec<f64> = (0..60).map(f64::from).collect();
        let result = describe(&values);
        assert_eq!(result["median_ms"], 29.5);
        assert_eq!(result["p10_ms"], 5.0);
        assert_eq!(result["p90_ms"], 53.0);
        assert_eq!(result["batch_medians_ms"], json!([9.5, 29.5, 49.5]));
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
    }
}
