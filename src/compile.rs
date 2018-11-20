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

pub fn main() -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    let success = compile(&lang)?;
    match success {
        true => Ok(()),
        false => Err(Error::new(ErrorKind::CompilationError())),
    }
}

pub fn compile(lang: &Lang) -> Result<bool> {
    if !compile::is_compile_needed(lang).unwrap_or(true) {
        print_info!(true, "no need to compile.");
        return Ok(true);
    }
    compile_src(lang).chain(ErrorKind::CompilationFailed())
}

pub fn compile_src(lang: &Lang) -> compile::Result<bool> {
    print_compiling!("{}", lang.src_file_name);
    let CompilerOutput {
        success,
        stdout,
        stderr,
    } = compile::compile(lang)?;
    print_compiler_output("standard output", stdout);
    print_compiler_output("standard error", stderr);

    Ok(success)
}

pub fn print_compiler_output(kind: &str, output: Option<String>) {
    if let Some(output) = output {
        let output = output.trim().split('\n');
        print_info!(true, "compiler {}:", kind);
        for line in output {
            colored_eprintln! {
                common::colorize();
                OUTPUT_COLOR, "        {}", line;
            }
        }
    }
}
