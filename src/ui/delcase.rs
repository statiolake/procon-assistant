use crate::imp::test_case;
use crate::imp::test_case::TestCase;
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

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to delete the testcase")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to open testcase file")]
    OpeningTestCaseFailed { source: anyhow::Error },

    #[error("the specified test case {idx} is out-of-range: there're only {num} test cases")]
    IndexOutOfRange { idx: i32, num: usize },

    #[error("failed to remove testcase file")]
    RemovingTestCaseFailed { source: anyhow::Error },

    #[error("failed to write testcase file into file")]
    WritingTestCaseFailed { source: anyhow::Error },
}

impl DelCase {
    pub fn run(self, _quiet: bool) -> Result<()> {
        // load all test cases
        let mut test_cases = test_case::enumerate_test_cases()
            .map_err(|e| Error(ErrorKind::OpeningTestCaseFailed { source: e.into() }))?;

        // once remove all test case file
        clean_test_cases(&test_cases)
            .map_err(|e| Error(ErrorKind::RemovingTestCaseFailed { source: e.into() }))?;

        // remove test case from test cases
        let len = test_cases.len();
        let mut removed = 0;
        #[allow(clippy::explicit_counter_loop)]
        for idx in self.indices {
            if idx >= len {
                return Err(Error(ErrorKind::IndexOutOfRange {
                    idx: (idx + 1) as _,
                    num: len,
                }));
            }

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
                .map_err(|e| Error(ErrorKind::WritingTestCaseFailed { source: e.into() }))?;
        }

        Ok(())
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
