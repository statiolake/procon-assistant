use crate::imp::common;
use crate::imp::langs::Lang;
use std::fs::File;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to compile")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to spawn compiler; check your installation")]
    SpawningCompilerFailed { source: anyhow::Error },

    #[error("failed to check the files metadata")]
    CheckingMetadataFailed { source: anyhow::Error },
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

pub fn compile(lang: &Lang) -> Result<CompilerOutput> {
    let result = (lang.compile_command_maker)(common::colorize())
        .output()
        .map_err(|e| Error(ErrorKind::SpawningCompilerFailed { source: e.into() }))?;

    let stdout = wrap_output_to_option(&result.stdout).map(output_to_string);
    let stderr = wrap_output_to_option(&result.stderr).map(output_to_string);

    Ok(CompilerOutput::new(result.status.success(), stdout, stderr))
}

pub fn is_compile_needed(lang: &Lang) -> Result<bool> {
    let to_err = |e: io::Error| Error(ErrorKind::CheckingMetadataFailed { source: e.into() });
    let src = File::open(&lang.src_file_name).map_err(to_err)?;
    let bin = File::open("main.exe").map_err(to_err)?;
    let src_modified = src.metadata().map_err(to_err)?.modified().map_err(to_err)?;
    let bin_modified = bin.metadata().map_err(to_err)?.modified().map_err(to_err)?;
    Ok(src_modified > bin_modified)
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
