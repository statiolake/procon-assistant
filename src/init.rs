use config::ConfigFile;
use std::env;
use std::fs::File;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use common;

use {Error, Result};

const FILES: &[&str] = &[
    ".clang_complete",
    "main.cpp",
    ".vscode/c_cpp_properties.json",
    ".vscode/tasks.json",
    ".vscode/launch.json",
];

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config()?;
    for file in FILES {
        let path = Path::new(file);
        if path.exists() {
            print_info!("file {} already exists, skipping.", file);
            continue;
        }

        generate(path).map_err(|e| {
            Error::with_cause(
                format!("generating {}", file),
                "failed to generate file.",
                box e,
            )
        })?;
        print_generated!("{}", file);
    }

    if config.auto_open {
        match config.open_directory_instead_of_specific_file {
            true => common::open(&config.editor, ".")?,
            false => common::open(&config.editor, "main.cpp")?,
        }
    }

    Ok(())
}

fn create_and_write_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path.as_ref())?;
    f.write_all(content.as_bytes())
}

fn generate(path: &Path) -> io::Result<()> {
    let exe_dir = env::current_exe()?;
    let exe_dir = exe_dir.parent().unwrap();
    let template_path = exe_dir
        .join("template")
        .join(&path.components().collect::<PathBuf>());
    print_info!("loading template from {}", template_path.display());
    let mut template_file = File::open(template_path)?;

    let mut content = String::new();
    template_file.read_to_string(&mut content)?;
    let content = content.replace("$LIB_DIR", &libdir_escaped());
    create_and_write_file(path, &content)
}

fn libdir_escaped() -> String {
    common::get_procon_lib_dir()
        .unwrap()
        .display()
        .to_string()
        .escape_default()
}
