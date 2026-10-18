use crate::evals::{Run, require, require_judgment};
use crate::support::Result;
use std::collections::BTreeSet;

pub fn check(run: &Run) -> Result<()> {
    require(
        run.trace.iter().any(|event| event["key"] == "version"),
        "Observe the version command",
    )?;
    let expected = run.snapshots["version"]
        .as_str()
        .ok_or("Missing observed version")?
        .trim()
        .trim_start_matches("sdev ");
    let versions: BTreeSet<&str> = run
        .response
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .filter(|part| {
            let components: Vec<&str> = part.split('.').collect();
            components.len() == 3
                && components.iter().all(|value| {
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                })
        })
        .collect();
    require(
        versions == BTreeSet::from([expected]),
        "Report the observed version, not another version number",
    )?;
    require_judgment(run)
}
