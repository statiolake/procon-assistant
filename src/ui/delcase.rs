use crate::imp::test_case;
use crate::imp::test_case::TestCase;
use crate::ExitStatus;
use anyhow::ensure;
use anyhow::{Context, Result};
use std::fs;
use std::io;

#[derive(clap::Clap)]
#[clap(
    about = "Deletes the specified test case;  removes `inX.txt` and `outX.txt`, and decrement the case number of succeeding test cases"
)]
pub struct DelCase {
    #[clap(help = "the list of test case numbers to remove")]
    indices: Vec<usize>,
}

impl DelCase {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        // load all test cases
        let mut test_cases =
            test_case::enumerate_test_cases().context("failed to enumerate test cases")?;

        // once remove all test case file
        clean_test_cases(&test_cases).context("failed to clean test cases")?;

        // remove test case from test cases
        let len = test_cases.len();
        let mut removed = 0;
        #[allow(clippy::explicit_counter_loop)]
        for idx in self.indices {
            ensure!(
                idx < len,
                "index is out of range: len is {} but idx is {}",
                len,
                idx
            );

            assert!(idx >= removed);
            test_cases.remove(idx - removed);
            removed += 1;
        }

        // re-generate test cases without removed cases
        for (idx, test_case) in test_cases.into_iter().enumerate() {
            let new_test_case = TestCase::new_with_idx(
                (idx + 1) as i32,
                test_case.if_contents,
                test_case.of_contents,
            );
            new_test_case
                .write()
                .context("failed to write to the test case")?;
        }

        Ok(ExitStatus::Success)
    }
}

fn clean_test_cases(test_cases: &[TestCase]) -> io::Result<()> {
    for test_case in test_cases {
        remove_test_case(test_case)?;
    }

    Ok(())
}

fn remove_test_case(test_case: &TestCase) -> io::Result<()> {
    fs::remove_file(&test_case.if_name)?;
    fs::remove_file(&test_case.of_name)?;

    Ok(())
}
