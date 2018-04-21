use std::path::Path;
use std::process::Command;

use {Error, Result};

pub struct SrcFile {
    pub file_name: String,
    pub compile_cmd: Command,
}

impl SrcFile {
    pub fn new(file_name: String, compile_cmd: Command) -> SrcFile {
        SrcFile {
            file_name,
            compile_cmd,
        }
    }
}

pub fn get_source_file() -> Result<SrcFile> {
    use config::src_support;
    for lang in src_support::LANGS {
        if Path::new(lang.src_file_name).exists() {
            let mut cmd = Command::new(lang.compiler);
            if let Some(modifier_func) = lang.cmd_pre_modifier {
                modifier_func(&mut cmd)?;
            }
            cmd.args(lang.flags);
            let file_name = lang.src_file_name.into();
            let compile_cmd = cmd;
            return Ok(SrcFile::new(file_name, compile_cmd));
        }
    }
    return Err(Error::new(
        "getting source file",
        "no supported source file found.",
    ));
}
