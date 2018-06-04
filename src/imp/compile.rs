use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::LightMagenta;
use imp::common;
use imp::srcfile;
use imp::srcfile::SrcFile;

const OUTPUT_COLOR: ConsoleColor = LightMagenta;

define_error!();
define_error_kind! {
    [GettingSourceFileFailed; (); "failed to get source file."];
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

pub fn compile() -> Result<CompilerOutput> {
    let SrcFile {
        file_name,
        mut compile_cmd,
    } = srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed())?;

    print_compiling!("{}", file_name);
    let result = compile_cmd
        .output()
        .chain(ErrorKind::SpawningCompilerFailed())?;

    let stdout = wrap_output_to_option(&result.stdout).map(output_to_string);
    let stderr = wrap_output_to_option(&result.stderr).map(output_to_string);

    Ok(CompilerOutput::new(result.status.success(), stdout, stderr))
}

pub fn print_compiler_output(kind: &str, output: Option<String>) {
    if let Some(output) = output {
        let output = output.trim().split('\n');
        print_info!(true, "compiler {}:", kind);
        for line in output {
            colored_println! {
                common::colorize();
                OUTPUT_COLOR, "        {}", line;
            }
        }
    }
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
