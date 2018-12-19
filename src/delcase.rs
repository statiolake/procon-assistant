use std::fs;
use std::io;

use crate::imp::test_case;
use crate::imp::test_case::TestCaseFile;

define_error!();
define_error_kind! {
    [ParsingCommandLineArgFailed; (); "failed to parse command line argument.".into()];
    [OpeningTestCaseFileFailed; (); "failed to open testcase file.".into()];
    [IndexOutOfRange; (idx: i32, num: usize); format!("the specified test case {} is out-of-range: there're only {} test cases.", idx, num)];
    [RemovingTestCaseFileFailed; (); "failed to remove testcase file.".into()];
    [WritingTestCaseFileFailed; (); "failed to write testcase file into file.".into()];
}

pub fn main(_quiet: bool, args: Vec<String>) -> Result<()> {
    let indices = parse_args(args)?;

    // load all test cases
    let mut test_cases: Vec<_> = test_case::enumerate_test_cases()
        .chain(ErrorKind::OpeningTestCaseFileFailed())?
        .into_iter()
        .map(|test_case| test_case.into_test_case_file())
        .collect();

    // once remove all test case file
    clean_test_cases(&test_cases).chain(ErrorKind::RemovingTestCaseFileFailed())?;

    // remove test case from test cases
    let len = test_cases.len();
    let mut removed = 0;
    #[allow(clippy::explicit_counter_loop)]
    for idx in indices {
        if idx >= len {
            return Err(Error::new(ErrorKind::IndexOutOfRange(
                (idx + 1) as i32,
                len,
            )));
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
            .chain(ErrorKind::WritingTestCaseFileFailed())?;
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
            .parse()
            .chain(ErrorKind::ParsingCommandLineArgFailed())?;

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
