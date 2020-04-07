use crate::eprintln_tagged;
use crate::imp::common;
use crate::imp::test_case;
use crate::ExitStatus;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Adds a new test case; creates `inX.txt` and `outX.txt` in the current directory")]
pub struct AddCase;

impl AddCase {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        // Create and write an empty test case
        let test_case = test_case::add_test_case(String::new(), String::new())
            .context("failed to create a new test case")?;
        eprintln_tagged!("Created": "{}, {}", test_case.if_name, test_case.of_name);

        common::open_addcase(&[&test_case.if_name, &test_case.of_name])
            .context("failed to open the generated file")?;

        Ok(ExitStatus::Success)
    }
}
