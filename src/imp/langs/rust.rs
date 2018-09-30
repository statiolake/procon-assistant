use super::Lang;
use super::Result;

use std::process::Command;

pub const LANG: Lang = Lang {
    file_type: "rust",
    src_file_name: "main.rs",
    compiler: "rustc",
    flags_setter: flags_setter,
};

pub fn flags_setter(cmd: &mut Command) -> Result<()> {
    cmd.args(&["main.rs"]);
    Ok(())
}
