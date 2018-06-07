pub mod judge_result;

use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use time;

use self::judge_result::{JudgeResult, OutputDifference};
use imp::common;
use imp::config;

define_error!();
define_error_kind! {
    [MismatchingTestCaseFiles; (existing_outfile: String, inexisting_infile: String); format!(
        "output file `{}' exists while input file `{}' does not exist.",
        existing_outfile, inexisting_infile
    )];
    [FileCreationFailed; (name: String); format!("failed to create `{}'", name)];
    [ExecutionOfMainBinaryFailed; (); format!("failed to execute compiled binary main.")];
}

#[derive(Debug)]
pub enum TestCase {
    File(TestCaseFile),
}

impl TestCase {
    pub fn judge(self) -> (String,  Result<(time::Duration, JudgeResult)>) {
        match self {
            TestCase::File(tcf) => (tcf.to_string(), tcf.judge()),
        }
    }
}

impl From<TestCaseFile> for TestCase {
    fn from(tcf: TestCaseFile) -> TestCase {
        TestCase::File(tcf)
    }
}

#[derive(Debug)]
pub struct TestCaseFile {
    pub if_name: String,
    pub if_contents: Vec<u8>,
    pub of_name: String,
    pub of_contents: Vec<u8>,
}

impl TestCaseFile {
    pub fn new(
        if_name: String,
        if_contents: Vec<u8>,
        of_name: String,
        of_contents: Vec<u8>,
    ) -> TestCaseFile {
        TestCaseFile {
            if_name,
            if_contents,
            of_name,
            of_contents,
        }
    }

    pub fn new_with_idx(idx: i32, if_contents: Vec<u8>, of_contents: Vec<u8>) -> TestCaseFile {
        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);
        TestCaseFile::new(if_name, if_contents, of_name, of_contents)
    }

    pub fn next_unused_idx() -> Result<i32> {
        let mut idx = 1;
        while Path::new(&make_if_name(idx)).exists() {
            idx += 1;
        }

        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);

        if Path::new(&of_name).exists() {
            return Err(Error::new(ErrorKind::MismatchingTestCaseFiles(
                of_name.into(),
                if_name.into(),
            )));
        }

        Ok(idx)
    }

    pub fn load_from(if_name: String, of_name: String) -> io::Result<TestCaseFile> {
        let mut if_contents = Vec::new();
        let mut of_contents = Vec::new();
        File::open(&if_name)?.read_to_end(&mut if_contents)?;
        File::open(&of_name)?.read_to_end(&mut of_contents)?;
        Ok(TestCaseFile::new(
            if_name,
            if_contents,
            of_name,
            of_contents,
        ))
    }

    pub fn load_from_index_of(idx: i32) -> io::Result<TestCaseFile> {
        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);
        TestCaseFile::load_from(if_name, of_name)
    }

    pub fn write(&self) -> Result<()> {
        common::ensure_to_create_file(&self.if_name, &self.if_contents)
            .chain(ErrorKind::FileCreationFailed(self.if_name.clone()))?;
        common::ensure_to_create_file(&self.of_name, &self.of_contents)
            .chain(ErrorKind::FileCreationFailed(self.of_name.clone()))?;
        Ok(())
    }

    pub fn judge(self) -> Result<(time::Duration, JudgeResult)> {
        let if_contents = remove_cr(self.if_contents);
        let mut child = spawn_main()?;
        input_to_child(&mut child, &if_contents);
        let (duration, maybe_judge) = wait_or_timeout(&mut child)?;
        if let Some(judge) = maybe_judge {
            return Ok((duration, judge));
        }

        let of_contents = remove_cr(self.of_contents);
        let childstdout = remove_cr(read_child_stdout(&mut child));

        if of_contents != childstdout {
            Ok((duration, compare_content_in_detail(
                if_contents,
                of_contents,
                childstdout,
            )))
        } else {
            Ok((duration, JudgeResult::Passed))
        }
    }
}

impl fmt::Display for TestCaseFile {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.if_name)
    }
}

pub fn enumerate_test_cases() -> io::Result<Vec<TestCase>> {
    let mut result = vec![];
    let mut i = 1;
    while Path::new(&make_if_name(i)).exists() {
        result.push(TestCase::from(TestCaseFile::load_from_index_of(i)?));
        i += 1;
    }

    Ok(result)
}

fn make_if_name(num: i32) -> String {
    format!("in{}.txt", num)
}

fn make_of_name(num: i32) -> String {
    format!("out{}.txt", num)
}

fn spawn_main() -> Result<Child> {
    Command::new("./main")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .chain(ErrorKind::ExecutionOfMainBinaryFailed())
}

fn input_to_child(child: &mut Child, if_contents: &Vec<u8>) {
    child.stdin.take().unwrap().write_all(if_contents).unwrap()
}

fn wait_or_timeout(child: &mut Child) -> Result<(time::Duration, Option<JudgeResult>)> {
    let start = time::now();
    let timeout_at = start + time::Duration::milliseconds(config::TIMEOUT_MILLISECOND);
    loop {
        let now = time::now();
        let try_wait_result = child.try_wait();
        let res = match try_wait_result {
            Ok(Some(status)) => (false, handle_try_wait_normal(status)),
            Ok(None) => (true, None),
            Err(_) => (false, handle_try_wait_error()),
        };
        match res {
            (_, Some(re)) => return Ok((now - start, Some(re))),
            (true, _) => {}
            (false, _) => break,
        }

        if timeout_at < now {
            // timeout!
            child.kill().unwrap();
            return Ok((now - start, Some(JudgeResult::TimeLimitExceeded)));
        }
    }
    let now = time::now();
    Ok((now - start, None))
}

fn handle_try_wait_normal(status: ExitStatus) -> Option<JudgeResult> {
    if status.code().is_none() {
        // signal termination. consider it as a runtime error here.
        Some(JudgeResult::RuntimeError(
            "process was terminated by a signal.".into(),
        ))
    } else if status.success() {
        // ok, child succesfully exited in time.
        None
    } else {
        // some error occurs, returning runtime error.
        Some(JudgeResult::RuntimeError(
            "exit status was not successful.".into(),
        ))
    }
}

fn handle_try_wait_error() -> Option<JudgeResult> {
    Some(JudgeResult::RuntimeError(
        "error occured while waiting process finish.".into(),
    ))
}

fn read_child_stdout(child: &mut Child) -> Vec<u8> {
    let mut childstdout = Vec::new();
    child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_end(&mut childstdout)
        .unwrap();
    childstdout
}

fn compare_content_in_detail(
    if_contents: Vec<u8>,
    of_contents: Vec<u8>,
    childstdout: Vec<u8>,
) -> JudgeResult {
    // wrong answer or presentation error
    let input = String::from_utf8_lossy(&if_contents);
    let input = input.trim().split('\n').map(|x| x.to_string()).collect();
    let expected_output = String::from_utf8_lossy(&of_contents).to_string();
    let expected_output = split_lines_to_vec(expected_output);
    let actual_output = String::from_utf8_lossy(&childstdout).to_string();
    let actual_output = split_lines_to_vec(actual_output);

    let difference = judge_result::enumerate_different_lines(&expected_output, &actual_output);
    return if difference == OutputDifference::NotDifferent {
        JudgeResult::PresentationError
    } else {
        JudgeResult::WrongAnswer(Some(judge_result::WrongAnswer {
            input,
            expected_output,
            actual_output,
            difference,
        }))
    };
}

fn split_lines_to_vec(s: String) -> Vec<String> {
    s.trim().split('\n').map(|x| x.to_string()).collect()
}

fn remove_cr(v: Vec<u8>) -> Vec<u8> {
    v.into_iter().filter(|&x| x != '\r' as u8).collect()
}
