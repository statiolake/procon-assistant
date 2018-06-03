use imp::compile;
use imp::compile::CompilerOutput;

define_error!();
define_error_kind! {
    [BuildingFailed; (); "build was not successful."];
    [ChildError; (); "during processing"];
}

pub fn main() -> Result<()> {
    let CompilerOutput {
        success,
        stdout,
        stderr,
    } = compile::compile().chain(ErrorKind::ChildError())?;
    compile::print_compiler_output("standard output", stdout);
    compile::print_compiler_output("standard error", stderr);
    match success {
        true => Ok(()),
        false => Err(Error::new(ErrorKind::BuildingFailed())),
    }
}
