use crate::imp::compile;
use crate::imp::compile::CompilerOutput;
use crate::imp::langs;
use crate::imp::langs::Lang;

#[derive(clap::Clap)]
#[clap(about = "Compiles the current solution;  the produced binary won't be tested automatically")]
pub struct Compile {
    #[clap(
        short,
        long,
        help = "Recompiles even if the compiled binary seems to be up-to-date"
    )]
    force: bool,
}

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to compile")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to run compilation task")]
    CompilationFailed { source: anyhow::Error },

    #[error("compilation was not successful: your code may have error")]
    CompilationError,

    #[error("failed to get source file")]
    GettingLanguageFailed { source: anyhow::Error },
}

impl Compile {
    pub fn run(self, quiet: bool) -> Result<()> {
        let lang = langs::get_lang()
            .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
        let success = compile(quiet, &lang, self.force)?;
        if success {
            Ok(())
        } else {
            Err(Error(ErrorKind::CompilationError))
        }
    }
}

pub fn compile(quiet: bool, lang: &Lang, force: bool) -> Result<bool> {
    if !force && !compile::is_compile_needed(lang).unwrap_or(true) {
        print_info!(!quiet, "no need to compile");
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
            eprintln!("        {}", line);
        }
    }
}
