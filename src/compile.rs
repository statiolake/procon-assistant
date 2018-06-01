use imp::compile;
use imp::compile::CompilerOutput;
use {Error, Result};

pub fn main() -> Result<()> {
    let CompilerOutput {
        success,
        stdout,
        stderr,
    } = compile::compile()?;
    compile::print_compiler_output("standard output", stdout);
    compile::print_compiler_output("standard error", stderr);
    match success {
        true => Ok(()),
        false => Err(Error::new("compiling", "build was not successful.")),
    }
}
