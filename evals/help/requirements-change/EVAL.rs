use crate::evals::{Run, require_judgment};
use crate::support::Result;

pub fn check(run: &Run) -> Result<()> {
    require_judgment(run)
}
