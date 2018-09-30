use std::process::Command;

use super::Lang;
use super::Result;
use imp::common;

pub const PROCON_LIB_DIR: &str = "procon-lib";

pub const LANG: Lang = Lang {
    file_type: "cpp",
    src_file_name: "main.cpp",
    compiler: "clang++",
    flags_setter: flags_setter,
};

pub fn flags_setter(cmd: &mut Command) -> Result<()> {
    cmd.arg(format!("-I{}", common::get_procon_lib_dir().display()).escape_default());
    cmd.args(&[
        "-g",
        "-std=c++14",
        "-Wall",
        "-Wextra",
        "-Wno-old-style-cast",
        "-DPA_DEBUG",
        "-omain",
        "main.cpp",
    ]);
    if cfg!(unix) {
        cmd.arg("-fdiagnostics-color=always");
    }
    Ok(())
}
