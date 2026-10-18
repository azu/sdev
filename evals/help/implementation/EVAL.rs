use crate::evals::{Run, check_sum, require_judgment};
use crate::support::Result;

pub fn check(run: &Run) -> Result<()> {
    check_sum(&run.directory)?;
    require_judgment(run)
}
