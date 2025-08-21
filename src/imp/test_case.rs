use crate::imp;
use crate::imp::config::CONFIG;
use anyhow::{anyhow, bail, ensure};
use anyhow::{Context as _, Result};
use itertools::izip;
use scopeguard::defer;
use std::borrow::Cow;
use std::cell::RefCell;
use std::io::{stdin, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::{cmp, fmt, fs, iter, time};
use wait_timeout::ChildExt;

pub struct JudgeResult {
    pub elapsed: time::Duration,
    pub result: TestResult,
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Accepted(Accepted),
    WrongAnswer(WrongAnswer),
    PresentationError(PresentationError),
    TimeLimitExceeded(TimeLimitExceeded),
    RuntimeError(RuntimeError),
    CompilationError,
}

impl TestResult {
    pub fn is_accepted(&self) -> bool {
        matches!(self, TestResult::Accepted(_))
    }

    pub fn is_failed(&self) -> bool {
        !self.is_accepted()
    }

    pub fn long_name(&self) -> &'static str {
        match self {
            TestResult::Accepted(_) => "Accepted",
            TestResult::WrongAnswer(_) => "Wrong Answer",
            TestResult::PresentationError(_) => "Presentation Error",
            TestResult::TimeLimitExceeded(_) => "Time Limit Exceeded",
            TestResult::RuntimeError(_) => "Runtime Error",
            TestResult::CompilationError => "Compilation Error",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            TestResult::Accepted(_) => "AC",
            TestResult::WrongAnswer(_) => "WA",
            TestResult::PresentationError(_) => "PE",
            TestResult::TimeLimitExceeded(_) => "TLE",
            TestResult::RuntimeError(_) => "RE",
            TestResult::CompilationError => "CE",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WrongAnswerKind {
    NumOfLineDiffers {
        expected: usize,
        actual: usize,
    },
    NumOfTokenDiffers {
        expected: usize,
        actual: usize,
        expected_span: Span,
        actual_span: Span,
    },
    TokenDiffers {
        expected: Token,
        actual: Token,
    },
}

impl fmt::Display for WrongAnswerKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WrongAnswerKind::NumOfLineDiffers { expected, actual } => write!(
                b,
                "The number of lines is different. expected: {}, actual: {}",
                expected, actual
            ),

            WrongAnswerKind::NumOfTokenDiffers {
                expected,
                actual,
                expected_span,
                actual_span,
            } => {
                assert_eq!(expected_span.line, actual_span.line);
                write!(
                    b,
                    "At line {}: the number of tokens is different. expected: {}, actual: {}",
                    expected_span.line + 1,
                    expected,
                    actual
                )
            }

            WrongAnswerKind::TokenDiffers { expected, actual } => {
                assert_eq!(expected.span.line, actual.span.line);
                write!(
                    b,
                    "At line {}: Token differs. expected: {}, actual: {}",
                    expected.span.line + 1,
                    expected,
                    actual,
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    SignalTerminated,
    ChildUnsuccessful,
    WaitingFinishFailed,
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeErrorKind::SignalTerminated => write!(b, "process was terminated by a signal"),
            RuntimeErrorKind::ChildUnsuccessful => write!(b, "exit status was not successful"),
            RuntimeErrorKind::WaitingFinishFailed => write!(b, "failed to wait the process finish"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Accepted {
    pub stderr: String,
}

impl Accepted {
    pub fn new_empty() -> Accepted {
        Accepted {
            stderr: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrongAnswer {
    pub context: Context,
    pub stderr: String,
    pub errors: Vec<WrongAnswerKind>,
}

#[derive(Debug, Clone)]
pub struct PresentationError {
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct TimeLimitExceeded {
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub stderr: String,
    pub kind: RuntimeErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub range: (usize, usize),
}

impl Span {
    pub fn start(&self) -> usize {
        self.range.0
    }

    pub fn end(&self) -> usize {
        self.range.1
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub expected: Vec<String>,
    pub actual: Vec<String>,
}

impl Context {
    pub fn new(expected: Vec<String>, actual: Vec<String>) -> Context {
        Context { expected, actual }
    }

    pub fn verify(mut self, stderr: String) -> TestResult {
        // fix last newline at first
        let is_presentation_error = self.check_presentation_error();
        self.fix_last_newline();

        // check the number of lines are equal. if it doesn't, it's already a wrong answer.
        if let Some(err) = Context::verify_num_lines(self.expected.len(), self.actual.len()) {
            return TestResult::WrongAnswer(WrongAnswer {
                context: self,
                stderr,
                errors: vec![err],
            });
        }

        // check each line and collect errors
        let mut errors = Vec::new();
        for (lineno, (expected, actual)) in izip!(&self.expected, &self.actual).enumerate() {
            let errors_line = Context::verify_line(expected, actual, lineno);
            errors.extend(errors_line);
        }

        // if there is any error, it's a wrong answer.
        if !errors.is_empty() {
            return TestResult::WrongAnswer(WrongAnswer {
                context: self,
                stderr,
                errors,
            });
        }

        if is_presentation_error {
            return TestResult::PresentationError(PresentationError { stderr });
        }

        TestResult::Accepted(Accepted { stderr })
    }

    /// Checks if it could be a presentation error. Check is meaningful only
    /// before calling `fix_last_newline()` as that function "fixes" the
    /// presentation error if any.
    fn check_presentation_error(&self) -> bool {
        // a newline is always needed just before EOF.
        self.actual.last().map(String::as_str) != Some("")
    }

    /// Fixes the presentation error if any.
    fn fix_last_newline(&mut self) {
        // if last line has a newline in the end, `expected` will have an
        // extra blank line. remove this.
        if self.expected.last().map(String::as_str) == Some("") {
            self.expected.pop();
        }

        // if the last line has a newline in the end, `actual` will have an
        // extra blank line. if it doesn't, that's a presentation error (last
        // newline is always required).
        if self.actual.last().map(String::as_str) == Some("") {
            self.actual.pop();
        }
    }

    fn verify_num_lines(expected: usize, actual: usize) -> Option<WrongAnswerKind> {
        if expected != actual {
            Some(WrongAnswerKind::NumOfLineDiffers { expected, actual })
        } else {
            None
        }
    }

    fn verify_line(expected_line: &str, actual_line: &str, lineno: usize) -> Vec<WrongAnswerKind> {
        let expected = Token::parse_line(expected_line, lineno);
        let actual = Token::parse_line(actual_line, lineno);

        // check the number of tokens
        if expected.len() != actual.len() {
            let expected_span = Span {
                line: lineno,
                range: (0, expected_line.len()),
            };

            let actual_span = Span {
                line: lineno,
                range: (0, actual_line.len()),
            };

            return vec![WrongAnswerKind::NumOfTokenDiffers {
                expected: expected.len(),
                actual: actual.len(),
                expected_span,
                actual_span,
            }];
        }

        // check for each token
        let mut errors = vec![];
        for (expected, actual) in expected.iter().zip(actual.iter()) {
            if !Token::is_equal(expected, actual) {
                errors.push(WrongAnswerKind::TokenDiffers {
                    expected: expected.clone(),
                    actual: actual.clone(),
                });
            }
        }

        errors
    }
}

fn split_into_lines(s: &str) -> impl Iterator<Item = &str> {
    s.split('\n').map(|x| x.trim_end_matches('\r'))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    fn new(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }

    /// Checks if two tokens are "equal". Note that this equality doesn't
    /// satisfy the transitivity (because a certain amount of error is allowed
    /// for floating point numbers).
    fn is_equal(a: &Token, b: &Token) -> bool {
        match (&a.kind, &b.kind) {
            (TokenKind::String(a), TokenKind::String(b)) => a == b,
            (TokenKind::Uint(a), TokenKind::Uint(b)) => a == b,
            (TokenKind::Int(a), TokenKind::Int(b)) => a == b,
            (TokenKind::Float(a), TokenKind::Float(b)) => (a - b).abs() < CONFIG.run.eps_for_float,
            _ => false,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            TokenKind::String(v) => write!(b, "{}", v),
            TokenKind::Uint(v) => write!(b, "{}", v),
            TokenKind::Int(v) => write!(b, "{}", v),
            TokenKind::Float(v) => write!(b, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    String(String),
    Uint(u64),
    Int(i64),
    Float(f64),
}

impl Token {
    fn parse_line(line: &str, line_no: usize) -> Vec<Token> {
        // if it contains two successive whitespace or starts with or ends with whitespace, treat
        // entire line as string literal.
        if line.contains("  ") || line.starts_with(' ') || line.ends_with(' ') {
            return vec![Token::new(
                TokenKind::String(line.into()),
                Span {
                    line: line_no,
                    range: (0, line.len()),
                },
            )];
        }

        // otherwise, split tokens and parse each token
        let mut iter = line.char_indices().peekable();

        iter::from_fn(|| match iter.peek() {
            None => None,
            Some(_) => {
                let word_char_indices = iter
                    .by_ref()
                    .skip_while(|(_, ch)| ch.is_whitespace())
                    .take_while(|(_, ch)| !ch.is_whitespace());
                let (start, end, token) = word_char_indices.fold(
                    (usize::MAX, usize::MIN, String::new()),
                    |(start, end, mut current), (column_no, ch)| {
                        current.push(ch);
                        let start = cmp::min(start, column_no);
                        let end = cmp::max(end, column_no + 1);
                        (start, end, current)
                    },
                );

                let span = Span {
                    line: line_no,
                    range: (start, end),
                };
                let token = Token::parse(&token, span);

                Some(token)
            }
        })
        .collect()
    }

    fn parse(token: &str, span: Span) -> Token {
        // A token starting with zero is rarely intended to be a number, so
        // treat it as a string. But there are some corner cases (ex: `0`,
        // `0.1`) so check that.
        if token.starts_with('0') && !(token == "0" || token.starts_with("0.")) {
            return Token::new(TokenKind::String(token.into()), span);
        }

        if let Ok(uint) = token.parse() {
            return Token::new(TokenKind::Uint(uint), span);
        }

        if let Ok(int) = token.parse() {
            return Token::new(TokenKind::Int(int), span);
        }

        if let Ok(float) = token.parse() {
            return Token::new(TokenKind::Float(float), span);
        }

        // falling back
        Token::new(TokenKind::String(token.into()), span)
    }
}

#[derive(Debug)]
pub enum TestCase {
    File(TestCaseFile),
    Stdin(TestCaseStdin),
}

impl TestCase {
    /// Judge the output of the specified command using this test case.
    pub fn judge(self, cmd: Command, timeout: Option<time::Duration>) -> Result<JudgeResult> {
        let timer = time::Instant::now();
        // spawn the solution
        let mut child = spawn(cmd)?;
        input_to_child(&mut child, &self.get_input()?)?;

        // wait for the solution to finish or timeout
        let (elapsed, maybe_result) = wait_or_timeout(timer, &mut child, timeout)?;
        let (stdout, stderr) = match maybe_result {
            WaitResult::Output(stdout, stderr) => (stdout, stderr),
            WaitResult::TestResult(result) => return Ok(JudgeResult { elapsed, result }),
        };

        // read the output
        let actual = split_into_lines(&stdout).map(ToString::to_string).collect();
        let expected = split_into_lines(&self.get_output()?)
            .map(ToString::to_string)
            .collect();
        let result = Context::new(expected, actual).verify(stderr);

        Ok(JudgeResult { elapsed, result })
    }

    pub fn get_input(&self) -> Result<Cow<'_, [u8]>> {
        match self {
            TestCase::File(f) => f.get_input(),
            TestCase::Stdin(s) => s.get_input(),
        }
    }

    pub fn get_output(&self) -> Result<Cow<'_, str>> {
        match self {
            TestCase::File(f) => f.get_output(),
            TestCase::Stdin(s) => s.get_output(),
        }
    }
}

#[derive(Debug)]
pub struct TestCaseFile {
    pub if_name: String,
    pub if_contents: String,
    pub of_name: String,
    pub of_contents: String,
}

impl TestCaseFile {
    pub fn new(
        if_name: String,
        if_contents: String,
        of_name: String,
        of_contents: String,
    ) -> TestCaseFile {
        TestCaseFile {
            if_name,
            if_contents,
            of_name,
            of_contents,
        }
    }

    pub fn new_with_idx(idx: i32, if_contents: String, of_contents: String) -> TestCaseFile {
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

        ensure!(
            !Path::new(&of_name).exists(),
            "mismatching test cases: `{}` exists but `{}` does not exist",
            of_name,
            if_name
        );

        Ok(idx)
    }

    pub fn load_from(if_name: String, of_name: String) -> Result<TestCaseFile> {
        let if_contents = fs::read_to_string(&if_name)?;
        let of_contents = fs::read_to_string(&of_name)?;

        Ok(TestCaseFile::new(
            if_name,
            if_contents,
            of_name,
            of_contents,
        ))
    }

    pub fn load_from_index(idx: i32) -> Result<TestCaseFile> {
        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);

        TestCaseFile::load_from(if_name, of_name)
    }

    pub fn write(&self) -> Result<()> {
        imp::fs::safe_write(&self.if_name, &self.if_contents)
            .with_context(|| format!("failed to create `{}`", self.if_name))?;
        imp::fs::safe_write(&self.of_name, &self.of_contents)
            .with_context(|| format!("failed to create `{}`", self.if_name))?;

        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        fs::remove_file(&self.if_name)?;
        fs::remove_file(&self.of_name)?;

        Ok(())
    }

    pub fn get_input(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::from(self.if_contents.as_bytes()))
    }

    pub fn get_output(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::from(&self.of_contents))
    }
}

#[derive(Debug)]
pub struct TestCaseStdin;

impl TestCaseStdin {
    pub fn get_input(&self) -> Result<Cow<'_, [u8]>> {
        let mut input = String::new();
        stdin()
            .read_to_string(&mut input)
            .context("failed to read from stdin")?;
        Ok(Cow::from(input.into_bytes()))
    }

    pub fn get_output(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::from("(no output can be specified for stdin)"))
    }
}

impl fmt::Display for TestCase {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestCase::File(f) => write!(b, "{}", f),
            TestCase::Stdin(_) => write!(b, "(stdin)"),
        }
    }
}

impl fmt::Display for TestCaseFile {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.if_name)
    }
}

pub fn enumerate_test_case_files() -> Result<Vec<TestCaseFile>> {
    let mut result = vec![];
    let mut i = 1;
    while Path::new(&make_if_name(i)).exists() {
        result.push(TestCaseFile::load_from_index(i)?);
        i += 1;
    }

    Ok(result)
}

pub fn add_test_case(if_contents: String, of_contents: String) -> Result<TestCaseFile> {
    let idx = TestCaseFile::next_unused_idx()?;
    let test_case = TestCaseFile::new_with_idx(idx, if_contents, of_contents);
    test_case.write()?;

    Ok(test_case)
}

/// Remove all specified test cases
pub fn remove_test_cases(indices: &[i32]) -> Result<()> {
    let test_case_files = RefCell::new(enumerate_test_case_files()?);

    // Make sure to restore test cases before exit
    defer! {
        for (idx, test_case) in test_case_files.borrow().iter().enumerate() {
            let idx = (idx + 1) as i32;
            let test_case = TestCaseFile::new_with_idx(
                idx,
                test_case.if_contents.clone(),
                test_case.of_contents.clone(),
            );
            let _ = test_case.write();
        }
    }

    // !! BE CAREFUL !! Remove all test cases.
    clean_test_cases(&test_case_files.borrow())?;

    let len = test_case_files.borrow().len() as i32;
    let mut removed = 0;
    let mut err_indices = Vec::new();
    for &idx1 in indices {
        // translate index into zero-origin
        let idx0 = idx1 - 1;

        if !(0..len).contains(&idx0) {
            // translate index into one-origin
            err_indices.push(idx1);
            continue;
        }

        assert!(idx0 >= removed);
        test_case_files.borrow_mut().remove((idx0 - removed) as _);
        removed += 1;
    }

    if !err_indices.is_empty() {
        bail!("some of indices are out of range: {:?}", err_indices);
    }

    Ok(())
}

fn clean_test_cases(test_cases: &[TestCaseFile]) -> Result<()> {
    for test_case in test_cases {
        test_case.remove()?;
    }

    Ok(())
}

fn make_if_name(num: i32) -> String {
    format!("in{}.txt", num)
}

fn make_of_name(num: i32) -> String {
    format!("out{}.txt", num)
}

fn spawn(mut cmd: Command) -> Result<Child> {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to execute main binary")
}

fn input_to_child(child: &mut Child, if_contents: &[u8]) -> Result<()> {
    child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to get stdin"))?
        .write_all(if_contents)
        .context("failed to write to stdin")
}

enum WaitResult {
    TestResult(TestResult),
    Output(String, String),
}

fn wait_or_timeout(
    timer: time::Instant,
    child: &mut Child,
    timeout: Option<time::Duration>,
) -> Result<(time::Duration, WaitResult)> {
    use self::TestResult::{RuntimeError as RE, TimeLimitExceeded as TLE};

    // thread to read stdout/stderr to prevent buffer to be full
    let (stdout_thread, stderr_thread) = {
        let mut child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to get child stdout"))?;

        let mut child_stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("failed to get child stderr"))?;

        let stdout_thread = thread::spawn(move || -> std::io::Result<String> {
            let mut stdout = String::new();
            child_stdout
                .read_to_string(&mut stdout)
                .map(move |_| stdout)
        });

        let stderr_thread = thread::spawn(move || -> std::io::Result<String> {
            let mut stderr = String::new();
            child_stderr
                .read_to_string(&mut stderr)
                .map(move |_| stderr)
        });

        (stdout_thread, stderr_thread)
    };

    let status = match timeout {
        Some(timeout) => child.wait_timeout(timeout).map_err(anyhow::Error::from),
        None => child.wait().map(Some).map_err(anyhow::Error::from),
    };

    match status {
        // child has somehow finished. check the reason.
        Ok(Some(status)) => {
            let result = if status.success() {
                // OK: child succesfully exited in time.
                let stdout = stdout_thread
                    .join()
                    .map_err(|_| anyhow!("failed to join stdout reader"))?
                    .context("failed to read stdout")?;
                let stderr = stderr_thread
                    .join()
                    .map_err(|_| anyhow!("failed to join stderr reader"))?
                    .context("failed to read stderr")?;
                WaitResult::Output(stdout, stderr)
            } else if status.code().is_none() {
                // RE: signal termination. consider it as a runtime error here.
                let _ = stdout_thread.join();
                let stderr = stderr_thread
                    .join()
                    .map_err(|_| anyhow!("failed to join stderr reader"))?
                    .context("failed to read stderr")?;
                WaitResult::TestResult(RE(RuntimeError {
                    stderr,
                    kind: RuntimeErrorKind::SignalTerminated,
                }))
            } else {
                // RE: some error occurs, returning runtime error.
                let _ = stdout_thread.join();
                let stderr = stderr_thread
                    .join()
                    .map_err(|_| anyhow!("failed to join stderr reader"))?
                    .context("failed to read stderr")?;
                WaitResult::TestResult(RE(RuntimeError {
                    stderr,
                    kind: RuntimeErrorKind::ChildUnsuccessful,
                }))
            };

            Ok((timer.elapsed(), result))
        }

        // child was TLE. continue to polling
        Ok(None) => {
            child.kill().unwrap();
            let _ = stdout_thread.join();
            let stderr = stderr_thread
                .join()
                .map_err(|_| anyhow!("failed to join stderr reader"))?
                .context("failed to read stderr")?;
            Ok((
                timer.elapsed(),
                WaitResult::TestResult(TLE(TimeLimitExceeded { stderr })),
            ))
        }

        // failed to check the child status. treat this as a runtime error.
        Err(_) => {
            let _ = stdout_thread.join();
            let stderr = stderr_thread
                .join()
                .map_err(|_| anyhow!("failed to join stderr reader"))?
                .context("failed to read stderr")?;
            Ok((
                timer.elapsed(),
                WaitResult::TestResult(RE(RuntimeError {
                    stderr,
                    kind: RuntimeErrorKind::WaitingFinishFailed,
                })),
            ))
        }
    }
}
