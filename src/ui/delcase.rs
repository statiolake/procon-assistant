use crate::imp::test_case;
use crate::ExitStatus;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(
    about = "Deletes the specified test case;  removes `inX.txt` and `outX.txt`, and decrement the case number of succeeding test cases"
)]
pub struct DelCase {
    #[clap(help = "the list of test case numbers to remove")]
    indices: Vec<i32>,
}

impl DelCase {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        test_case::remove_test_cases(&self.indices).context("failed to remove some test cases")?;
        Ok(ExitStatus::Success)
    }
}
