use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;
use std::thread;

use clip;
use common;
use config;
use imp::compile;
use imp::compile::CompilerOutput;
use imp::srcfile;
use imp::srcfile::SrcFile;
use imp::test_case;
use imp::test_case::judge_result::{JudgeResult, WrongAnswer};
use imp::test_case::{TestCase, TestCaseFile};

const OUTPUT_COLOR: ConsoleColor = LightMagenta;

define_error!();
define_error_kind! {
    [CompilationFailed; (); format!("failed to compile.")];
    [RunningTestsFailed; (); format!("test running failed.")];
    [GettingSourceFileFailed; (); format!("failed to get source file.")];
    [CopyingToClipboardFailed; (); format!("failed to copy to clipboard.")];
    [InvalidArgument; (); format!("failed to parse the passed argument.")];
    [InexistingTestCase; (n: i32); format!("testcase {} doesn't exist.", n)];
    [JudgingFailed; (); format!("failed to judge.")];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let CompilerOutput {
        success,
        stdout,
        stderr,
    } = compile::compile().chain(ErrorKind::CompilationFailed())?;

    compile::print_compiler_output("standard output", stdout);
    compile::print_compiler_output("standard error", stderr);
    let result = match success {
        true => run_tests(&args).chain(ErrorKind::RunningTestsFailed())?,
        false => JudgeResult::CompilationError,
    };

    let (result_color, result_long_verb, result_long_name) = result.long_name();
    println!("");
    colored_println!{
        common::colorize();
        Reset, "    Your solution {}", result_long_verb;
        result_color, "{}", result_long_name;
        Reset, ".";
    };

    // copy the answer to the clipboard
    if let JudgeResult::Passed = result {
        let SrcFile { file_name, .. } =
            srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed())?;
        println!("");
        clip::copy_to_clipboard(file_name.as_ref()).chain(ErrorKind::CopyingToClipboardFailed())?;
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
        let tcf = TestCaseFile::new_with_index(n);
        if !tcf.exists() {
            return Err(Error::new(ErrorKind::InexistingTestCase(n)));
        }
        result.push(TestCase::from(tcf));
    }

    Ok(result)
}

fn enumerate_test_cases(args: &Vec<String>) -> Result<Vec<TestCase>> {
    let test_cases = if args.is_empty() {
        test_case::enumerate_test_cases()
    } else {
        parse_argument_cases(args)?
    };

    Ok(test_cases)
}

fn print_solution_output(kind: &str, result: &Vec<String>) {
    print_info!("{}:", kind);
    for line in result.iter() {
        colored_println! {
            common::colorize();
            OUTPUT_COLOR, "        {}", line;
        }
    }
}

fn run(tcs: Vec<TestCase>) -> Result<JudgeResult> {
    print_running!(
        "{} testcases (current timeout is {} millisecs)",
        tcs.len(),
        config::TIMEOUT_MILLISECOND
    );

    let handles: Vec<_> = tcs.into_iter()
        .map(|tc| thread::spawn(move || tc.judge()))
        .collect();

    // `map` is lazy evaluated so join() is not executed here unless they are
    // collected to Vec. if not, `Finished running` is instantly displayed
    // regardless of judging finished or not.
    let judge_results: Vec<_> = handles.into_iter().map(|x| x.join().unwrap()).collect();

    print_finished!("running");
    println!("");
    let mut whole_result = JudgeResult::Passed;
    for (display, result) in judge_results.into_iter() {
        let result = result.chain(ErrorKind::JudgingFailed())?;
        print_result(&result, display);
        // update whole result
        if result != JudgeResult::Passed && whole_result == JudgeResult::Passed {
            whole_result = result;
        }
    }
    Ok(whole_result)
}

fn print_result(result: &JudgeResult, display: String) {
    // get color and short result string
    let (color, short_name) = result.short_name();
    colored_println! {
        common::colorize();
        Reset, "    ";
        color, "{}", short_name;
        Reset, " {}", display;
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
            print_info!("{}", difference.message());
        }
        JudgeResult::RuntimeError(ref reason) => {
            print_info!("{}", reason);
        }
        _ => {}
    }
}
