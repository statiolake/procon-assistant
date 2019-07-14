use crate::imp::common;
use crate::imp::compile;
use crate::imp::compile::CompilerOutput;
use crate::imp::langs;
use crate::imp::langs::Lang;
use colored_print::color::{ConsoleColor, ConsoleColor::Reset};
use colored_print::colored_eprintln;

const OUTPUT_COLOR: ConsoleColor = Reset;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to compile")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to run compilation task.")]
    CompilationFailed { source: anyhow::Error },

    #[error("compilation was not successful: your code may have error.")]
    CompilationError,

    #[error("failed to get source file.")]
    GettingLanguageFailed { source: anyhow::Error },
}

pub fn main(quiet: bool, args: Vec<String>) -> Result<()> {
    let lang = langs::get_lang()
        .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
    let force = check_force_paremeter(&args);
    let success = compile(quiet, &lang, force)?;
    if success {
        Ok(())
    } else {
        Err(Error(ErrorKind::CompilationError))
    }
}

pub fn compile(quiet: bool, lang: &Lang, force: bool) -> Result<bool> {
    if !force && !compile::is_compile_needed(lang).unwrap_or(true) {
        print_info!(!quiet, "no need to compile.");
        return Ok(true);
    }
    compile_src(lang).map_err(|e| Error(ErrorKind::CompilationFailed { source: e.into() }))
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

fn check_force_paremeter(args: &[String]) -> bool {
    args.iter().any(|x| x == "--force" || x == "-f")
}
