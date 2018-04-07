use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use std::result;

use common;

use Error;
use Result;

const TIMEOUT_MILLISECOND: i64 = 1000;
const OUTPUT_COLOR: ConsoleColor = LightMagenta;

#[derive(PartialEq, Eq, Clone, Debug)]
enum JudgeResult {
    Passed,
    WrongAnswer(Option<(Vec<String>, Vec<String>, Vec<String>, OutputDifference)>), // in, expected, actual, different lines
    PresentationError,
    TimeLimitExceeded,
    RuntimeError(String), // reason
    CompilationError,
}

impl JudgeResult {
    fn to_long_name(&self) -> (ConsoleColor, &'static str) {
        use self::JudgeResult::*;
        match *self {
            Passed => (Green, "Sample Case Passed"),
            WrongAnswer(_) => (Yellow, "Wrong Answer"),
            PresentationError => (Yellow, "Presentation Error"),
            TimeLimitExceeded => (Yellow, "Time Limit Exceeded"),
            RuntimeError(_) => (Red, "Runtime Error"),
            CompilationError => (Yellow, "Compilation Error"),
        }
    }

    fn to_short_name(&self) -> (ConsoleColor, &'static str) {
        use self::JudgeResult::*;
        match *self {
            Passed => (Green, "PS "),
            WrongAnswer(_) => (Yellow, "WA "),
            PresentationError => (Yellow, "PE "),
            TimeLimitExceeded => (Yellow, "TLE"),
            RuntimeError(_) => (Red, "RE "),
            CompilationError => (Yellow, "CE "),
        }
    }
}

fn print_compiler_output(kind: &str, output: &Vec<u8>) {
    if !output.is_empty() {
        let output = String::from_utf8_lossy(output);
        let output = output.trim();
        let output = output.split('\n');
        print_info!("compiler {}:", kind);
        for line in output {
            colored_println! {
                common::colorize();
                OUTPUT_COLOR, "        {}", line;
            }
        }
    }
}

fn compile() -> result::Result<bool, String> {
    print_compiling!("main.cpp");
    let result = Command::new("g++")
        .arg("-std=c++14")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-omain")
        .arg("main.cpp")
        .output()
        .map_err(|x| {
            format!(
                "failed to spawn g++: {}. check you instlaled g++ correctly.",
                x
            )
        })?;

    print_compiler_output("standard output", &result.stdout);
    print_compiler_output("standard error", &result.stderr);

    Ok(result.status.success())
}

type Filenames = Vec<(String, String)>;

fn check_exists(filenames: &Filenames) -> result::Result<(), String> {
    for filename in filenames.iter() {
        let (ref infile_name, ref outfile_name) = *filename;
        if !Path::new(infile_name).exists() {
            return Err(format!("{} does not exist.", infile_name));
        }
        if !Path::new(outfile_name).exists() {
            return Err(format!("{} does not exist.", outfile_name));
        }
    }

    Ok(())
}

fn get_current_dirs_cases() -> Filenames {
    let mut result = vec![];
    let mut i = 1;
    while Path::new(&common::make_infile_name(i)).exists() {
        let infile_name = common::make_infile_name(i);
        let outfile_name = common::make_outfile_name(i);
        result.push((infile_name, outfile_name));
        i += 1;
    }

    result
}

fn parse_argument_cases(args: &Vec<String>) -> result::Result<Filenames, String> {
    let mut result = vec![];
    for arg in args.iter() {
        let n: i32 = arg.parse()
            .map_err(|x| format!("failed to parse argument: {}", x))?;
        let infile_name = common::make_infile_name(n);
        let outfile_name = common::make_outfile_name(n);
        result.push((infile_name, outfile_name));
    }

    Ok(result)
}

fn enumerate_filenames(args: &Vec<String>) -> result::Result<Filenames, String> {
    let filenames = if args.is_empty() {
        get_current_dirs_cases()
    } else {
        parse_argument_cases(args)?
    };

    check_exists(&filenames)?;
    Ok(filenames)
}

fn run_for_one_file(infile_name: &str, outfile_name: &str) -> result::Result<JudgeResult, String> {
    // get infile content
    let mut infile = File::open(infile_name).unwrap();
    let mut infile_content = Vec::new();
    infile.read_to_end(&mut infile_content).unwrap();

    // spawn executable
    let mut child = Command::new("./main")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|x| format!("failed to spawn main: {}", x))?;

    // pipe infile content into child stdin
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&infile_content)
        .unwrap();

    // checking loop: during current time become timeout_time, pooling the child aliving status.
    let timeout_time = ::time::now() + ::time::Duration::milliseconds(TIMEOUT_MILLISECOND);
    loop {
        let try_wait_result = child.try_wait();
        if let Ok(Some(status)) = try_wait_result {
            if status.code().is_none() {
                // signal termination. consider it as a runtime error here.
                return Ok(JudgeResult::RuntimeError(
                    "process was terminated by a signal.".into(),
                ));
            }
            if status.success() {
                // ok, child succesfully exited in time.
                break;
            } else {
                // some error occurs, returning runtime error.
                return Ok(JudgeResult::RuntimeError(
                    "exit status was not successful.".into(),
                ));
            }
        } else if let Ok(None) = try_wait_result {
            // running
        } else if let Err(_) = try_wait_result {
            // some error occurs, returning runtime error.
            return Ok(JudgeResult::RuntimeError(
                "error occured while waiting process finish.".into(),
            ));
        }

        if timeout_time < ::time::now() {
            // timeout!
            child.kill().unwrap();
            return Ok(JudgeResult::TimeLimitExceeded);
        }
    }

    // read outfile content
    let mut outfile = File::open(outfile_name).unwrap();
    let mut outfile_content = Vec::new();
    outfile.read_to_end(&mut outfile_content).unwrap();

    // read child stdout
    let mut childstdout = Vec::new();
    child.stdout.unwrap().read_to_end(&mut childstdout).unwrap();

    // when they don't match:
    if childstdout != outfile_content {
        // wrong answer or presentation error
        let infile = String::from_utf8_lossy(&infile_content);
        let infile = infile.trim().split('\n').map(|x| x.to_string()).collect();
        let expected = String::from_utf8_lossy(&outfile_content);
        let expected = expected.trim().split('\n').map(|x| x.to_string()).collect();
        let actual = String::from_utf8_lossy(&childstdout);
        let actual = actual.trim().split('\n').map(|x| x.to_string()).collect();
        let difference = enumerate_different_lines(&expected, &actual);
        return if difference == OutputDifference::NotDifferent {
            Ok(JudgeResult::PresentationError)
        } else {
            Ok(JudgeResult::WrongAnswer(Some((
                infile,
                expected,
                actual,
                difference,
            ))))
        };
    }

    // otherwise, they matches (the solution is accepted).
    Ok(JudgeResult::Passed)
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

#[derive(PartialEq, Eq, Clone, Debug)]
enum OutputDifference {
    SizeDiffers,
    Different(Vec<usize>),
    NotDifferent,
}

impl OutputDifference {
    fn message(&self) -> String {
        match *self {
            OutputDifference::SizeDiffers => format!("the number of output lines is different."),
            OutputDifference::NotDifferent => unreachable!(), // this should be treated as Presentation Error.
            OutputDifference::Different(ref different_lines) => {
                let message = different_lines
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(&", ".to_string());
                format!("line {} differs.", message)
            }
        }
    }
}

fn enumerate_different_lines(expected: &Vec<String>, actual: &Vec<String>) -> OutputDifference {
    if expected.len() != actual.len() {
        return OutputDifference::SizeDiffers;
    }

    let mut different_lines = vec![];
    for i in 0..expected.len() {
        if expected[i] != actual[i] {
            different_lines.push(i + 1);
        }
    }

    if different_lines.is_empty() {
        // this is not wrong answer, but maybe presentation error;
        OutputDifference::NotDifferent
    } else {
        OutputDifference::Different(different_lines)
    }
}

fn run(filenames: Filenames) -> result::Result<JudgeResult, String> {
    print_running!(
        "{} testcases (current timeout is {} millisecs)",
        filenames.len(),
        TIMEOUT_MILLISECOND
    );
    let handles: Vec<_> = filenames
        .into_iter()
        .map(|filename| {
            thread::spawn(move || {
                let (infile_name, outfile_name) = filename;
                let judge = run_for_one_file(&infile_name, &outfile_name);
                (infile_name, outfile_name, judge)
            })
        })
        .collect();

    // if don't collect, its just save the iterator, and join() is not executed here
    // (will be executed in `for` loop). so then `Finished running` is instantly
    // displayed regardless of judging finished or not.
    let judge_results: Vec<_> = handles.into_iter().map(|x| x.join().unwrap()).collect();

    print_finished!("running");
    println!("");
    let mut whole_result = JudgeResult::Passed;
    for (infile_name, _, result) in judge_results.into_iter() {
        let result = result?;

        // get color and short result string
        let (color, short_name) = result.to_short_name();
        colored_println! {
            common::colorize();
            Reset, "    ";
            color, "{}", short_name;
            Reset, " {}", infile_name;
        }

        match result {
            JudgeResult::WrongAnswer(Some((
                ref infile,
                ref expected,
                ref actual,
                ref difference,
            ))) => {
                print_solution_output("sample case input", &infile);
                print_solution_output("expected output", &expected);
                print_solution_output("actual output", &actual);
                print_info!("{}", difference.message());
            }
            JudgeResult::RuntimeError(ref reason) => {
                print_info!("{}", reason);
            }
            _ => {}
        }

        // update whole result
        if result != JudgeResult::Passed && whole_result == JudgeResult::Passed {
            whole_result = result;
        }
    }
    Ok(whole_result)
}

pub fn main(args: Vec<String>) -> Result<()> {
    let result = match compile() {
        Err(msg) => return Err(Error::new("compiling", msg)),
        Ok(b) if !b => JudgeResult::CompilationError,
        _ => enumerate_filenames(&args)
            .map_err(|e| Error::new("enumerating filenames", e))
            .and_then(|filenames| {
                run(filenames).map_err(|msg| Error::new("running testcase", msg))
            })?,
    };

    let (result_color, result_long_name) = result.to_long_name();
    println!("");
    colored_println!{
        common::colorize();
        Reset, "    Your solution was ";
        result_color, "{}", result_long_name;
        Reset, ".";
    };

    if let JudgeResult::Passed = result {
        let mut main_cpp_content = String::new();
        File::open("main.cpp")
            .unwrap()
            .read_to_string(&mut main_cpp_content)
            .unwrap();

        // copy content into clipboard
        let resultchild = Command::new("xsel").arg("-b").stdin(Stdio::piped()).spawn();
        if let Ok(mut child) = resultchild {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(main_cpp_content.as_bytes())
                .unwrap();
            child.wait().unwrap();
        }
    }

    Ok(())
}
