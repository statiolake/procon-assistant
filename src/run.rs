use colored_print::color::{ConsoleColor, ConsoleColor::*};
use colored_print::colored_eprintln;
use time;

use std::thread;

use crate::clip;
use crate::compile;
use crate::imp::common;
use crate::imp::config;
use crate::imp::langs;
use crate::imp::test_case;
use crate::imp::test_case::judge_result::{JudgeResult, WrongAnswer};
use crate::imp::test_case::{TestCase, TestCaseFile};

const OUTPUT_COLOR: ConsoleColor = LightMagenta;

define_error!();
define_error_kind! {
    [CompilationFailed; (); format!("failed to compile.")];
    [RunningTestsFailed; (); format!("test running failed.")];
    [GettingLanguageFailed; (); format!("failed to get source file.")];
    [CopyingToClipboardFailed; (); format!("failed to copy to clipboard.")];
    [InvalidArgument; (); format!("failed to parse the passed argument.")];
    [LoadingTestCaseFailed; (); format!("failed to load some test case.")];
    [JudgingFailed; (); format!("failed to judge.")];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    let success = compile::compile(&lang).chain(ErrorKind::CompilationFailed())?;
    let result = match success {
        true => run_tests(&args).chain(ErrorKind::RunningTestsFailed())?,
        false => JudgeResult::CompilationError,
    };

    let (result_color, result_long_verb, result_long_name) = result.long_name();
    eprintln!("");
    colored_eprintln!{
        common::colorize();
        Reset, "    Your solution {}", result_long_verb;
        result_color, "{}", result_long_name;
        Reset, ".";
    };

    // copy the answer to the clipboard
    if let JudgeResult::Passed = result {
        eprintln!("");
        clip::copy_to_clipboard(&lang).chain(ErrorKind::CopyingToClipboardFailed())?;
    }

    Ok(())
}

fn run_tests(args: &Vec<String>) -> Result<JudgeResult> {
    enumerate_test_cases(&args).and_then(|tcs| run(tcs))
}

fn parse_argument_cases(args: &Vec<String>) -> Result<Vec<TestCase>> {
    let mut result = vec![];
    for arg in args.iter() {
        let n: i32 = arg.parse().chain(ErrorKind::InvalidArgument())?;
        let tcf = TestCaseFile::load_from_index_of(n).chain(ErrorKind::LoadingTestCaseFailed())?;
        result.push(TestCase::from(tcf));
    }

    Ok(result)
}

fn enumerate_test_cases(args: &Vec<String>) -> Result<Vec<TestCase>> {
    let test_cases = if args.is_empty() {
        test_case::enumerate_test_cases().chain(ErrorKind::LoadingTestCaseFailed())
    } else {
        parse_argument_cases(args)
    }?;

    Ok(test_cases)
}

fn print_solution_output(kind: &str, result: &Vec<String>) {
    print_info!(true, "{}:", kind);
    for line in result.iter() {
        colored_eprintln! {
            common::colorize();
            OUTPUT_COLOR, "        {}", line;
        }
    }
}

fn run(tcs: Vec<TestCase>) -> Result<JudgeResult> {
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
        let (duration, result) = result.chain(ErrorKind::JudgingFailed())?;
        print_result(&result, &duration, display);
        // update whole result
        if result != JudgeResult::Passed && whole_result == JudgeResult::Passed {
            whole_result = result;
        }
    }
    Ok(whole_result)
}

fn print_result(result: &JudgeResult, duration: &time::Duration, display: String) {
    // get color and short result string
    let (color, short_name) = result.short_name();
    colored_eprintln! {
        common::colorize();
        Reset, "    ";
        color, "{}", short_name;
        Reset, " {}", display;
        Reset, " (in {} ms)", duration.num_milliseconds();
    }

    match result {
        JudgeResult::WrongAnswer(Some(WrongAnswer {
            ref input,
            ref expected_output,
            ref actual_output,
            ref difference,
        })) => {
            print_solution_output("sample case input", &input);
            print_solution_output("expected output", &expected_output);
            print_solution_output("actual output", &actual_output);
            print_info!(true, "{}", difference.message());
        }
        JudgeResult::RuntimeError(ref reason) => {
            print_info!(true, "{}", reason);
        }
        _ => {}
    }
}
