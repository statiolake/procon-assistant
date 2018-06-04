use imp::srcfile::SrcFile;

define_error!();
define_error_kind! {
    [SpawningCompilerFailed; (); "failed to spawn compiler; check your installation."];
}

pub struct CompilerOutput {
    pub success: bool,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl CompilerOutput {
    pub fn new(success: bool, stdout: Option<String>, stderr: Option<String>) -> CompilerOutput {
        CompilerOutput {
            success,
            stdout,
            stderr,
        }
    }
}

pub fn compile(mut src: SrcFile) -> Result<CompilerOutput> {
    let result = src.compile_cmd
        .output()
        .chain(ErrorKind::SpawningCompilerFailed())?;

    let stdout = wrap_output_to_option(&result.stdout).map(output_to_string);
    let stderr = wrap_output_to_option(&result.stderr).map(output_to_string);

    Ok(CompilerOutput::new(result.status.success(), stdout, stderr))
}

fn wrap_output_to_option(output: &[u8]) -> Option<(&[u8])> {
    match output.is_empty() {
        true => None,
        false => Some(output),
    }
}

fn output_to_string(output: &[u8]) -> String {
    if cfg!(windows) {
        // for windows, decode output as cp932 (a.k.a. windows 31j)
        use encoding::all::WINDOWS_31J;
        use encoding::{DecoderTrap, Encoding};
        WINDOWS_31J
            .decode(output, DecoderTrap::Strict)
            .unwrap_or("(failed to decode output)".into())
    } else {
        // otherwise, decode output as utf-8 as usual.
        String::from_utf8_lossy(output).into()
    }
}
