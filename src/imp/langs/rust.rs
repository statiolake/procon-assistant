use std::path::PathBuf;
use std::process::Command;

use super::Lang;
use imp::common;
use imp::preprocess;

pub const LANG: Lang = Lang {
    file_type: "rust",
    src_file_name: "main.rs",
    compiler: "rustc",
    lib_dir_getter: get_lib_dir,
    compile_command_maker: compile_command,
    preprocessor: preprocess::rust::preprocess,
    minifier: preprocess::rust::minify,
};

fn compile_command() -> Command {
    let mut cmd = Command::new(LANG.compiler);
    flags_setter(&mut cmd);
    cmd
}

fn get_lib_dir() -> PathBuf {
    let mut home_dir = common::get_home_path();
    home_dir.push("procon-lib-rs");
    home_dir
}

fn flags_setter(cmd: &mut Command) {
    cmd.args(&["main.rs"]);
}
