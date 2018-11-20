use std::path::PathBuf;
use std::process::Command;

use super::Lang;
use crate::imp::common;
use crate::imp::preprocess;

pub const LANG: Lang = Lang {
    file_type: "cpp",
    src_file_name: "main.cpp",
    compiler: "g++",
    lib_dir_getter: get_lib_dir,
    compile_command_maker: compile_command,
    preprocessor: preprocess::cpp::preprocess,
    minifier: preprocess::cpp::minify,
};

fn compile_command() -> Command {
    let mut cmd = Command::new(LANG.compiler);
    flags_setter(&mut cmd);
    cmd
}

fn get_lib_dir() -> PathBuf {
    let mut home_dir = common::get_home_path();
    home_dir.push("procon-lib");
    home_dir
}

fn flags_setter(cmd: &mut Command) {
    cmd.arg(format!("-I{}", get_lib_dir().display()).escape_default());
    cmd.args(&[
        "-g",
        "-std=c++14",
        "-Wall",
        "-Wextra",
        "-Wno-old-style-cast",
        "-DPA_DEBUG",
        if cfg!(unix) { "-omain" } else { "-omain.exe" },
        "main.cpp",
    ]);
    if cfg!(unix) {
        cmd.arg("-fsanitize=address");
        cmd.arg("-fdiagnostics-color=always");
    }
}
