use crate::imp::langs::Language;
use crate::ExitStatus;
use anyhow::anyhow;
use anyhow::{Context, Result};

pub struct CompilerOutput {
    pub status: ExitStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl CompilerOutput {
    pub fn new(
        status: ExitStatus,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> CompilerOutput {
        CompilerOutput {
            status,
            stdout,
            stderr,
        }
    }
}

pub fn compile<L: Language + ?Sized>(lang: &L) -> Result<CompilerOutput> {
    let result = lang
        .compile_command()
        .into_iter()
        .map(|mut cmd| cmd.output())
        .last()
        .ok_or_else(|| anyhow!("compile command is empty"))?
        .context("failed to spawn compiler")?;

    let stdout = wrap_output_to_option(&result.stdout).map(output_to_string);
    let stderr = wrap_output_to_option(&result.stderr).map(output_to_string);

    let status = if result.status.success() {
        ExitStatus::Success
    } else {
        ExitStatus::Failure
    };

    Ok(CompilerOutput::new(status, stdout, stderr))
}

fn wrap_output_to_option(output: &[u8]) -> Option<&[u8]> {
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

fn output_to_string(output: &[u8]) -> String {
    if cfg!(windows) {
        // for windows, decode output as cp932 (a.k.a. windows 31j)
        use encoding::all::WINDOWS_31J;
        use encoding::{DecoderTrap, Encoding};
        WINDOWS_31J
            .decode(output, DecoderTrap::Strict)
            .unwrap_or_else(|_| "(failed to decode output)".into())
    } else {
        // otherwise, decode output as utf-8 as usual.
        String::from_utf8_lossy(output).into()
    }
}
