mod judge_result;

use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use std::result;

use self::judge_result::{JudgeResult, OutputDifference};
use common;
use config;
use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;
use Error;
use Result;

const OUTPUT_COLOR: ConsoleColor = LightMagenta;

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

fn compile() -> Result<result::Result<bool, String>> {
    let SrcFile {
        file_name,
        mut compile_cmd,
    } = srcfile::get_source_file()?;

    print_compiling!("{}", file_name);
    let result = compile_cmd.output().map_err(|x| {
        Error::new(
            "compiling source",
            format!(
                "failed to spawn g++: {}. check you instlaled g++ correctly.",
                x
            ),
        )
    })?;

    print_compiler_output("standard output", &result.stdout);
    print_compiler_output("standard error", &result.stderr);

    Ok(Ok(result.status.success()))
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
    let timeout_time = ::time::now() + ::time::Duration::milliseconds(config::TIMEOUT_MILLISECOND);
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
        let input = String::from_utf8_lossy(&infile_content);
        let input = input.trim().split('\n').map(|x| x.to_string()).collect();
        let expected_output = String::from_utf8_lossy(&outfile_content);
        let expected_output = expected_output
            .trim()
            .split('\n')
            .map(|x| x.to_string())
            .collect();
        let actual_output = String::from_utf8_lossy(&childstdout);
        let actual_output = actual_output
            .trim()
            .split('\n')
            .map(|x| x.to_string())
            .collect();
        let difference = judge_result::enumerate_different_lines(&expected_output, &actual_output);
        return if difference == OutputDifference::NotDifferent {
            Ok(JudgeResult::PresentationError)
        } else {
            Ok(JudgeResult::WrongAnswer(Some(judge_result::WrongAnswer {
                input,
                expected_output,
                actual_output,
                difference,
            })))
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

fn run(filenames: Filenames) -> result::Result<JudgeResult, String> {
    print_running!(
        "{} testcases (current timeout is {} millisecs)",
        filenames.len(),
        config::TIMEOUT_MILLISECOND
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
        let (color, short_name) = result.short_name();
        colored_println! {
            common::colorize();
            Reset, "    ";
            color, "{}", short_name;
            Reset, " {}", infile_name;
        }

        match result {
            JudgeResult::WrongAnswer(Some(judge_result::WrongAnswer {
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

        // update whole result
        if result != JudgeResult::Passed && whole_result == JudgeResult::Passed {
            whole_result = result;
        }
    }
    Ok(whole_result)
}

pub fn main(args: Vec<String>) -> Result<()> {
    let result = match compile()? {
        Err(msg) => return Err(Error::new("compiling", msg)),
        Ok(b) if !b => JudgeResult::CompilationError,
        _ => enumerate_filenames(&args)
            .map_err(|e| Error::new("enumerating filenames", e))
            .and_then(|filenames| {
                run(filenames).map_err(|msg| Error::new("running testcase", msg))
            })?,
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
        let SrcFile { file_name, .. } = srcfile::get_source_file()?;
        println!("");
        clip::copy_to_clipboard(file_name.as_ref())?;
    }

    Ok(())
}
