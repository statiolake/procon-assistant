use crate::imp;
use crate::imp::config::CONFIG;
use anyhow::{anyhow, bail, ensure};
use anyhow::{Context as _, Result};
use itertools::izip;
use scopeguard::defer;
use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::usize;
use std::{cmp, fmt, fs, iter, time};

pub struct JudgeResult {
    pub elapsed: time::Duration,
    pub result: TestResult,
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Accepted,
    WrongAnswer(WrongAnswer),
    PresentationError,
    TimeLimitExceeded,
    RuntimeError(RuntimeError),
    CompilationError,
}

impl TestResult {
    pub fn is_accepted(&self) -> bool {
        matches!(self, TestResult::Accepted)
    }

    pub fn is_failed(&self) -> bool {
        !self.is_accepted()
    }

    pub fn long_name(&self) -> &'static str {
        match self {
            TestResult::Accepted => "Accepted",
            TestResult::WrongAnswer(_) => "Wrong Answer",
            TestResult::PresentationError => "Presentation Error",
            TestResult::TimeLimitExceeded => "Time Limit Exceeded",
            TestResult::RuntimeError(_) => "Runtime Error",
            TestResult::CompilationError => "Compilation Error",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            TestResult::Accepted => "AC",
            TestResult::WrongAnswer(_) => "WA",
            TestResult::PresentationError => "PE",
            TestResult::TimeLimitExceeded => "TLE",
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
pub struct WrongAnswer {
    pub context: Context,
    pub stderr: String,
    pub errors: Vec<WrongAnswerKind>,
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
            return TestResult::PresentationError;
        }

        TestResult::Accepted
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
pub struct TestCase {
    pub if_name: String,
    pub if_contents: String,
    pub of_name: String,
    pub of_contents: String,
}

impl TestCase {
    pub fn new(
        if_name: String,
        if_contents: String,
        of_name: String,
        of_contents: String,
    ) -> TestCase {
        TestCase {
            if_name,
            if_contents,
            of_name,
            of_contents,
        }
    }

    pub fn new_with_idx(idx: i32, if_contents: String, of_contents: String) -> TestCase {
        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);
        TestCase::new(if_name, if_contents, of_name, of_contents)
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

    pub fn load_from(if_name: String, of_name: String) -> Result<TestCase> {
        let if_contents = fs::read_to_string(&if_name)?;
        let of_contents = fs::read_to_string(&of_name)?;

        Ok(TestCase::new(if_name, if_contents, of_name, of_contents))
    }

    pub fn load_from_index(idx: i32) -> Result<TestCase> {
        let if_name = make_if_name(idx);
        let of_name = make_of_name(idx);

        TestCase::load_from(if_name, of_name)
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

    /// Judge the output of the specified command using this test case.
    pub fn judge(self, cmd: Command) -> Result<JudgeResult> {
        // spawn the solution
        let mut child = spawn(cmd)?;
        input_to_child(&mut child, self.if_contents.as_bytes())?;

        // wait for the solution to finish or timeout
        let (elapsed, maybe_result) = wait_or_timeout(&mut child)?;
        if let Some(result) = maybe_result {
            return Ok(JudgeResult { elapsed, result });
        }

        // read the output
        let actual = split_into_lines(&read_child_stdout(&mut child))
            .map(ToString::to_string)
            .collect();
        let expected = split_into_lines(&self.of_contents)
            .map(ToString::to_string)
            .collect();
        let stderr = read_child_stderr(&mut child);
        let result = Context::new(expected, actual).verify(stderr);

        Ok(JudgeResult { elapsed, result })
    }
}

impl fmt::Display for TestCase {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.if_name)
    }
}

pub fn enumerate_test_cases() -> Result<Vec<TestCase>> {
    let mut result = vec![];
    let mut i = 1;
    while Path::new(&make_if_name(i)).exists() {
        result.push(TestCase::load_from_index(i)?);
        i += 1;
    }

    Ok(result)
}

pub fn add_test_case(if_contents: String, of_contents: String) -> Result<TestCase> {
    let idx = TestCase::next_unused_idx()?;
    let test_case = TestCase::new_with_idx(idx, if_contents, of_contents);
    test_case.write()?;

    Ok(test_case)
}

/// Remove all specified test cases
pub fn remove_test_cases(indices: &[i32]) -> Result<()> {
    let test_cases = RefCell::new(enumerate_test_cases()?);
    // Make sure to restore test cases before exit
    defer! {
        for (idx, test_case) in test_cases.borrow().iter().enumerate() {
            let idx = (idx + 1) as i32;
            let test_case = TestCase::new_with_idx(
                idx,
                test_case.if_contents.clone(),
                test_case.of_contents.clone(),
            );
            let _ = test_case.write();
        }
    }

    // !! BE CAREFUL !! Remove all test cases.
    clean_test_cases(&*test_cases.borrow())?;

    let len = test_cases.borrow().len() as i32;
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
        test_cases.borrow_mut().remove((idx0 - removed) as _);
        removed += 1;
    }

    if !err_indices.is_empty() {
        bail!("some of indices are out of range: {:?}", err_indices);
    }

    Ok(())
}

fn clean_test_cases(test_cases: &[TestCase]) -> Result<()> {
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

fn wait_or_timeout(child: &mut Child) -> Result<(time::Duration, Option<TestResult>)> {
    use self::TestResult::{RuntimeError as RE, TimeLimitExceeded as TLE};

    let timeout = time::Duration::from_millis(CONFIG.run.timeout_milliseconds);
    let timer = time::Instant::now();
    loop {
        // current elapsed time
        let elapsed = timer.elapsed();

        // check if the binary has finished.
        let try_wait_result = child.try_wait();
        match try_wait_result {
            // child has somehow finished. check the reason.
            Ok(Some(status)) => {
                let test_result = if status.success() {
                    // OK: child succesfully exited in time.
                    None
                } else if status.code().is_none() {
                    // RE: signal termination. consider it as a runtime error here.
                    let stderr = read_child_stderr(child);
                    Some(RE(RuntimeError {
                        stderr,
                        kind: RuntimeErrorKind::SignalTerminated,
                    }))
                } else {
                    // RE: some error occurs, returning runtime error.
                    let stderr = read_child_stderr(child);
                    Some(RE(RuntimeError {
                        stderr,
                        kind: RuntimeErrorKind::ChildUnsuccessful,
                    }))
                };

                return Ok((elapsed, test_result));
            }

            // child hasn't finished. continue to polling
            Ok(None) => {}

            // failed to check the child status. treat this as a runtime error.
            Err(_) => {
                let stderr = read_child_stderr(child);
                let test_result = Some(RE(RuntimeError {
                    stderr,
                    kind: RuntimeErrorKind::WaitingFinishFailed,
                }));
                return Ok((elapsed, test_result));
            }
        }

        if elapsed >= timeout {
            // timeout.
            child.kill().unwrap();
            return Ok((elapsed, Some(TLE)));
        }
    }
}

fn read_child_stdout(child: &mut Child) -> String {
    let mut childstdout = String::new();
    let stdout = child.stdout.as_mut().unwrap();
    stdout.read_to_string(&mut childstdout).unwrap();

    childstdout
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut childstderr = String::new();
    let stderr = child.stderr.as_mut().unwrap();
    stderr.read_to_string(&mut childstderr).unwrap();

    childstderr
}
