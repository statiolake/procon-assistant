use super::Lang;
use crate::imp::fs;
use crate::imp::preprocess;
use std::path::PathBuf;
use std::process::Command;

pub const LANG: Lang = Lang {
    lang: "rust",
    src_file_name: "main.rs",
    compiler: "rustc",
    lib_dir_getter: get_lib_dir,
    compile_command_maker: compile_command,
    preprocessor: preprocess::rust::preprocess,
    minifier: preprocess::rust::minify,
    linter: |_, _| Vec::new(),
};

fn compile_command(colorize: bool) -> Command {
    let mut cmd = Command::new(LANG.compiler);

    let colorize = if colorize { "always" } else { "never" };

    flags_setter(&mut cmd);
    cmd.arg("--color").arg(colorize);

    cmd
}

fn get_lib_dir() -> PathBuf {
    let mut home_dir = fs::get_home_path();
    home_dir.push("procon-lib-rs");
    home_dir
}

fn flags_setter(cmd: &mut Command) {
    cmd.args(&["main.rs"]);
}
