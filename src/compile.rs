use colored_print::color::{ConsoleColor, ConsoleColor::Reset};
use colored_print::colored_eprintln;

use crate::imp::common;
use crate::imp::compile;
use crate::imp::compile::CompilerOutput;
use crate::imp::langs;
use crate::imp::langs::Lang;

const OUTPUT_COLOR: ConsoleColor = Reset;

define_error!();
define_error_kind! {
    [CompilationFailed; (); "failed to run compilation task."];
    [CompilationError; (); "compilation was not successful: your code may have error."];
    [GettingLanguageFailed; (); "failed to get source file."];
}

pub fn main(quiet: bool, args: Vec<String>) -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    let force = check_force_paremeter(&args);
    let success = compile(quiet, &lang, force)?;
    if success {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::CompilationError()))
    }
}

pub fn compile(quiet: bool, lang: &Lang, force: bool) -> Result<bool> {
    if !force && !compile::is_compile_needed(lang).unwrap_or(true) {
        print_info!(!quiet, "no need to compile.");
        return Ok(true);
    }
    compile_src(quiet, lang).chain(ErrorKind::CompilationFailed())
}

pub fn compile_src(quiet: bool, lang: &Lang) -> compile::Result<bool> {
    print_compiling!("{}", lang.src_file_name);
    let CompilerOutput {
        success,
        stdout,
        stderr,
    } = compile::compile(lang)?;
    print_compiler_output(quiet, "standard output", stdout);
    print_compiler_output(quiet, "standard error", stderr);

    Ok(success)
}

pub fn print_compiler_output(quiet: bool, kind: &str, output: Option<String>) {
    if let Some(output) = output {
        let output = output.trim().split('\n');
        if !quiet {
            print_info!(!quiet, "compiler {}:", kind);
            for line in output {
                colored_eprintln! {
                    common::colorize();
                    OUTPUT_COLOR, "        {}", line;
                }
            }
        }
    }
}

fn check_force_paremeter(args: &[String]) -> bool {
    args.iter().any(|x| x == "--force" || x == "-f")
}
