use std::path::PathBuf;
use std::process::Command;

use super::Lang;
use crate::imp::common;
use crate::imp::preprocess;

pub const LANG: Lang = Lang {
    lang: "cpp",
    src_file_name: "main.cpp",
    compiler: "clang++",
    lib_dir_getter: get_lib_dir,
    compile_command_maker: compile_command,
    preprocessor: preprocess::cpp::preprocess,
    minifier: preprocess::cpp::minify,
    linter,
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
        #[cfg(windows)]
        "-gcodeview",
        "-O0",
        #[cfg(unix)]
        "-fdiagnostics-color=always",
        #[cfg(unix)]
        "-fsanitize=address,leak,memory,undefined",
        #[cfg(windows)]
        "-Xclang",
        #[cfg(windows)]
        "-flto-visibility-public-std",
        #[cfg(windows)]
        "-fno-delayed-template-parsing",
        "-std=c++14",
        "-Wall",
        "-Wextra",
        "-Wno-old-style-cast",
        "-DPA_DEBUG",
        #[cfg(unix)]
        "-omain",
        #[cfg(windows)]
        "-omain.exe",
        "main.cpp",
    ]);
}

fn linter(_quiet: bool, minified: &preprocess::Minified) -> Vec<String> {
    let minified = minified.inner();

    let mut result = Vec::new();
    if minified.contains("cerr") {
        result.push("cerr found.".into());
    }

    result
}
