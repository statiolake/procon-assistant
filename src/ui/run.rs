use crate::imp::common;
use crate::imp::config;
use crate::imp::langs;
use crate::imp::test_case;
use crate::imp::test_case::judge_result::{JudgeResult, WrongAnswer};
use crate::imp::test_case::{TestCase, TestCaseFile};
use crate::ui::clip;
use crate::ui::compile;
use colored_print::color::{ConsoleColor, ConsoleColor::*};
use colored_print::colored_eprintln;
use std::thread;
use std::time;

#[derive(clap::Clap)]
#[clap(about = "Runs and tests the current solution")]
pub struct Run {
    #[clap(
        short,
        long,
        help = "Recompiles even if the compiled binary seems to be up-to-date"
    )]
    force_compile: bool,
    #[clap(help = "Test case IDs to test")]
    to_run: Vec<String>,
}

const OUTPUT_COLOR: ConsoleColor = LightMagenta;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to run some test cases")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to compile")]
    CompilationFailed { source: anyhow::Error },

    #[error("test running failed")]
    RunningTestsFailed { source: anyhow::Error },

    #[error("failed to get source file")]
    GettingLanguageFailed { source: anyhow::Error },

    #[error("failed to copy to clipboard")]
    CopyingToClipboardFailed { source: anyhow::Error },

    #[error("failed to parse the passed argument")]
    InvalidArgument { source: anyhow::Error },

    #[error("failed to load some test case")]
    LoadingTestCaseFailed { source: anyhow::Error },

    #[error("failed to judge")]
    JudgingFailed { source: anyhow::Error },

    #[error("some of tests didn't pass")]
    TestCaseNotPass,
}

impl Run {
    pub fn run(self, quiet: bool) -> Result<()> {
        let lang = langs::get_lang()
            .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
        let success = compile::compile(quiet, &lang, self.force_compile)
            .map_err(|e| Error(ErrorKind::CompilationFailed { source: e.into() }))?;
        let result = if success {
            run_tests(quiet, &self.to_run)
                .map_err(|e| Error(ErrorKind::RunningTestsFailed { source: e.into() }))?
        } else {
            JudgeResult::CompilationError
        };

        let (result_color, result_long_verb, result_long_name) = result.long_name();
        eprintln!("");
        colored_eprintln! {
            common::colorize();
            Reset, "    Your solution {}", result_long_verb;
            result_color, "{}", result_long_name;
            Reset, "";
        };

        // copy the answer to the clipboard
        if let JudgeResult::Passed = result {
            eprintln!("");
            clip::copy_to_clipboard(quiet, &lang)
                .map_err(|e| Error(ErrorKind::CopyingToClipboardFailed { source: e.into() }))?;

            Ok(())
        } else {
            Err(Error(ErrorKind::TestCaseNotPass))
        }
    }
}

fn run_tests(quiet: bool, args: &[String]) -> Result<JudgeResult> {
    enumerate_test_cases(&args).and_then(|tcs| run(quiet, tcs))
}

fn parse_argument_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let mut result = vec![];
    for arg in args.iter() {
        let n: i32 = arg
            .parse::<i32>()
            .map_err(|e| Error(ErrorKind::InvalidArgument { source: e.into() }))?;
        let tcf = TestCaseFile::load_from_index_of(n)
            .map_err(|e| Error(ErrorKind::LoadingTestCaseFailed { source: e.into() }))?;
        result.push(TestCase::from(tcf));
    }

    Ok(result)
}

fn enumerate_test_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let test_cases = if args.is_empty() {
        test_case::enumerate_test_cases()
            .map_err(|e| Error(ErrorKind::LoadingTestCaseFailed { source: e.into() }))?
    } else {
        parse_argument_cases(args)?
    };

    Ok(test_cases)
}

fn print_solution_output(quiet: bool, kind: &str, result: &[String]) {
    print_info!(!quiet, "{}:", kind);
    for line in result.iter() {
        colored_eprintln! {
            common::colorize();
            OUTPUT_COLOR, "        {}", line;
        }
    }
}

fn run(quiet: bool, tcs: Vec<TestCase>) -> Result<JudgeResult> {
    print_running!(
        "{} test cases (current timeout is {} millisecs)",
        tcs.len(),
        config::TIMEOUT_MILLISECOND
    );

    let handles: Vec<_> = tcs
        .into_iter()
        .map(|tc| thread::spawn(move || tc.judge()))
        .collect();

    // `map` is lazy evaluated so join() is not executed here unless they are
    // collected to Vec. if not, `Finished running` is instantly displayed
    // regardless of judging finished or not.
    let judge_results: Vec<_> = handles.into_iter().map(|x| x.join().unwrap()).collect();

    print_finished!("running");
    eprintln!("");
    let mut whole_result = JudgeResult::Passed;
    for (display, result) in judge_results.into_iter() {
        let (duration, result) =
            result.map_err(|e| Error(ErrorKind::JudgingFailed { source: e.into() }))?;
        print_result(quiet, &result, &duration, display);
        // update whole result
        if result != JudgeResult::Passed && whole_result == JudgeResult::Passed {
            whole_result = result;
        }
    }
    Ok(whole_result)
}

fn print_result(quiet: bool, result: &JudgeResult, duration: &time::Duration, display: String) {
    // get color and short result string
    let (color, short_name) = result.short_name();
    colored_eprintln! {
        common::colorize();
        Reset, "    ";
        color, "{}", short_name;
        Reset, " {}", display;
        Reset, " (in {} ms)", duration.as_millis();
    }

    match result {
        JudgeResult::WrongAnswer(Some(WrongAnswer {
            ref input,
            ref expected_output,
            ref actual_output,
            ref difference,
        })) => {
            print_solution_output(quiet, "sample case input", &input);
            print_solution_output(quiet, "expected output", &expected_output);
            print_solution_output(quiet, "actual output", &actual_output);
            print_info!(!quiet, "{}", difference.message());
        }
        JudgeResult::RuntimeError(ref reason) => {
            print_info!(!quiet, "{}", reason);
        }
        _ => {}
    }
}
