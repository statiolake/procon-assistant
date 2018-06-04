pub mod judge_result;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use time;

use self::judge_result::{JudgeResult, OutputDifference};
use config;
use imp::common;

define_error!();
define_error_kind! {
    [MismatchingTestCaseFiles; (existing_outfile: String, inexisting_infile: String); format!(
        "output file `{}' exists while input file `{}' does not exist.",
        existing_outfile, inexisting_infile
    )];
    [FileCreationFailed; (name: String); format!(
        "failed to create `{}'", name
    )];
    [ExecutionOfMainBinaryFailed; (); format!(
        "failed to execute compiled binary main."
    )];
}

#[derive(Debug)]
pub enum TestCase {
    File(TestCaseFile),
}

impl TestCase {
    pub fn judge(&self) -> (String, Result<JudgeResult>) {
        match *self {
            TestCase::File(ref tcf) => (tcf.display().into(), tcf.judge()),
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
    pub infile_name: String,
    pub outfile_name: String,
}

impl TestCaseFile {
    pub fn new(infile_name: String, outfile_name: String) -> TestCaseFile {
        TestCaseFile {
            infile_name,
            outfile_name,
        }
    }

    pub fn new_with_index(idx: i32) -> TestCaseFile {
        TestCaseFile::new(make_infile_name(idx), make_outfile_name(idx))
    }

    pub fn new_with_next_unused_name() -> Result<TestCaseFile> {
        let mut i = 1;
        while Path::new(&make_infile_name(i)).exists() {
            i += 1;
        }

        let infile_name = make_infile_name(i);
        let outfile_name = make_outfile_name(i);

        if Path::new(&outfile_name).exists() {
            return Err(Error::new(ErrorKind::MismatchingTestCaseFiles(
                outfile_name.into(),
                infile_name.into(),
            )));
        }

        Ok(TestCaseFile::new(infile_name, outfile_name))
    }

    pub fn create(&self) -> Result<()> {
        self.create_with_contents("", "")
    }

    pub fn create_with_contents<S, T>(&self, if_contents: S, of_contents: T) -> Result<()>
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        common::ensure_to_create_file(&self.infile_name, if_contents.as_ref())
            .chain(ErrorKind::FileCreationFailed(self.infile_name.clone()))?;
        common::ensure_to_create_file(&self.outfile_name, of_contents.as_ref())
            .chain(ErrorKind::FileCreationFailed(self.outfile_name.clone()))?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.infile_name).exists() && Path::new(&self.outfile_name).exists()
    }

    pub fn display(&self) -> &str {
        &self.infile_name
    }

    pub fn judge(&self) -> Result<JudgeResult> {
        let if_contents = load_file(&self.infile_name);
        let mut child = spawn_main()?;
        input_to_child(&mut child, &if_contents);
        if let Some(judge) = wait_or_timeout(&mut child)? {
            return Ok(judge);
        }

        fn remove_cr(v: Vec<u8>) -> Vec<u8> {
            v.into_iter().filter(|x| *x != '\r' as u8).collect()
        }

        let of_contents = remove_cr(load_file(&self.outfile_name));
        let childstdout = remove_cr(read_child_stdout(&mut child));

        if of_contents != childstdout {
            Ok(compare_content_in_detail(
                if_contents,
                of_contents,
                childstdout,
            ))
        } else {
            Ok(JudgeResult::Passed)
        }
    }
}

pub fn enumerate_test_cases() -> Vec<TestCase> {
    let mut result = vec![];
    let mut i = 1;
    while Path::new(&make_infile_name(i)).exists() {
        let infile_name = make_infile_name(i);
        let outfile_name = make_outfile_name(i);
        result.push(TestCase::File(TestCaseFile::new(infile_name, outfile_name)));
        i += 1;
    }

    result
}

fn make_infile_name(num: i32) -> String {
    format!("in{}.txt", num)
}

fn make_outfile_name(num: i32) -> String {
    format!("out{}.txt", num)
}

fn load_file(file_name: &str) -> Vec<u8> {
    let mut contents = Vec::new();
    File::open(file_name)
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    contents
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

fn wait_or_timeout(child: &mut Child) -> Result<Option<JudgeResult>> {
    let timeout_at = time::now() + time::Duration::milliseconds(config::TIMEOUT_MILLISECOND);
    loop {
        let try_wait_result = child.try_wait();
        let res = match try_wait_result {
            Ok(Some(status)) => (false, handle_try_wait_normal(status)),
            Ok(None) => (true, None),
            Err(_) => (false, handle_try_wait_error()),
        };
        match res {
            (_, Some(re)) => return Ok(Some(re)),
            (true, _) => {}
            (false, _) => break,
        }

        if timeout_at < time::now() {
            // timeout!
            child.kill().unwrap();
            return Ok(Some(JudgeResult::TimeLimitExceeded));
        }
    }
    Ok(None)
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
