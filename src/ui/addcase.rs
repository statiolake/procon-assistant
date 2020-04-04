use crate::eprintln_tagged;
use crate::imp::common;
use crate::imp::test_case::TestCase;
use crate::ExitStatus;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Adds a new test case;  creates `inX.txt` and `outX.txt` in the current directory")]
pub struct AddCase;

impl AddCase {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        // Create empty test case
        let idx = TestCase::next_unused_idx().context("failed to get unused index")?;
        let tsf = TestCase::new_with_idx(idx, String::new(), String::new());

        // Write empty test case into file
        tsf.write().context("failed to write the test case")?;

        eprintln_tagged!("Created": "{}, {}", tsf.if_name, tsf.of_name);

        common::open_addcase(&[&tsf.if_name, &tsf.of_name])
            .context("failed to open the generated file")?;

        Ok(ExitStatus::Success)
    }
}
