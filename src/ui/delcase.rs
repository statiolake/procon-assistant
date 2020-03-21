use crate::imp::test_case;
use crate::imp::test_case::{TestCase, TestCaseFile};
use std::fs;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to delete the testcase")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to parse command line argument.")]
    ParsingCommandLineArgFailed { source: anyhow::Error },

    #[error("failed to open testcase file.")]
    OpeningTestCaseFileFailed { source: anyhow::Error },

    #[error("the specified test case {idx} is out-of-range: there're only {num} test cases.")]
    IndexOutOfRange { idx: i32, num: usize },

    #[error("failed to remove testcase file.")]
    RemovingTestCaseFileFailed { source: anyhow::Error },

    #[error("failed to write testcase file into file.")]
    WritingTestCaseFileFailed { source: anyhow::Error },
}

pub fn main(_quiet: bool, args: Vec<String>) -> Result<()> {
    let indices = parse_args(args)?;

    // load all test cases
    let mut test_cases: Vec<_> = test_case::enumerate_test_cases()
        .map_err(|e| Error(ErrorKind::OpeningTestCaseFileFailed { source: e.into() }))?
        .into_iter()
        .map(TestCase::into_test_case_file)
        .collect();

    // once remove all test case file
    clean_test_cases(&test_cases)
        .map_err(|e| Error(ErrorKind::RemovingTestCaseFileFailed { source: e.into() }))?;

    // remove test case from test cases
    let len = test_cases.len();
    let mut removed = 0;
    #[allow(clippy::explicit_counter_loop)]
    for idx in indices {
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
        let new_test_case = TestCaseFile::new_with_idx(
            (idx + 1) as i32,
            test_case.if_contents,
            test_case.of_contents,
        );
        new_test_case
            .write()
            .map_err(|e| Error(ErrorKind::WritingTestCaseFileFailed { source: e.into() }))?;
    }

    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Vec<usize>> {
    let mut res = Vec::new();
    for arg in args {
        // actually it can be `let num: usize`, and then succeeding cast won't
        // be needed. but, since I treat the index of test_case as i32 (refer to
        // TestCaseFile struct), the number should be parsed as i32 here.
        let num: i32 = arg
            .parse::<i32>()
            .map_err(|e| Error(ErrorKind::ParsingCommandLineArgFailed { source: e.into() }))?;

        res.push((num - 1) as usize);
    }

    Ok(res)
}

fn clean_test_cases(test_cases: &[TestCaseFile]) -> io::Result<()> {
    for test_case in test_cases {
        remove_test_case(test_case)?;
    }

    Ok(())
}

fn remove_test_case(test_case: &TestCaseFile) -> io::Result<()> {
    fs::remove_file(&test_case.if_name)?;
    fs::remove_file(&test_case.of_name)?;

    Ok(())
}
