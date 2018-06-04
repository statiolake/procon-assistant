use std::path::Path;
use std::process::Command;

define_error!();
define_error_kind! {
    [FileNotFound; (); "there seems not to have supported file in current dir."];
    [ChildError; (); "during processing"];
}

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
    use imp::config::src_support;
    for lang in src_support::LANGS {
        if Path::new(lang.src_file_name).exists() {
            let mut cmd = Command::new(lang.compiler);
            (lang.flags_setter)(&mut cmd).chain(ErrorKind::ChildError())?;
            let file_name = lang.src_file_name.into();
            let compile_cmd = cmd;
            return Ok(SrcFile::new(file_name, compile_cmd));
        }
    }
    return Err(Error::new(ErrorKind::FileNotFound()));
}
