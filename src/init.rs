use config::ConfigFile;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use common;

const FILES: &[&str] = &[
    ".clang_complete",
    "main.cpp",
    ".vscode/c_cpp_properties.json",
    ".vscode/tasks.json",
    ".vscode/launch.json",
];

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); format!("failed to get config.")];
    [CreateDestinationDirectoryFailed; (name: String); format!("creating directory `{}' failed.", name)];
    [CreateDestinationFailed; (name: String); format!("creating `{}' failed.", name)];
    [WriteToDestinationFailed; (name: String); format!("writing `{}' failed.", name)];
    [OpenTemplateFailed; (name: String); format!("template file for `{}' not found.", name)];
    [ReadFromTemplateFailed; (name: String); format!("reading from template `{}' failed.", name)];
    [OpeningEditorFailed; (); format!("failed to open editor.")];
}

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;
    for file in FILES {
        let path = Path::new(file);
        if path.exists() {
            print_info!(true, "file {} already exists, skipping.", file);
            continue;
        }

        generate(path)?;
        print_generated!("{}", file);
    }

    if config.auto_open {
        match config.open_directory_instead_of_specific_file {
            true => common::open(&config.editor, ".").chain(ErrorKind::OpeningEditorFailed())?,
            false => {
                common::open(&config.editor, "main.cpp").chain(ErrorKind::OpeningEditorFailed())?
            }
        }
    }

    Ok(())
}

fn generate(path: &Path) -> Result<()> {
    let exe_dir = env::current_exe().unwrap();
    let exe_dir = exe_dir.parent().unwrap();
    let template_path = exe_dir
        .join("template")
        .join(&path.components().collect::<PathBuf>());

    let template_path_string = template_path.display().to_string();
    print_info!(true, "loading template from `{}'", template_path_string);
    let mut template_file = File::open(template_path)
        .chain(ErrorKind::OpenTemplateFailed(template_path_string.clone()))?;

    let mut content = String::new();
    template_file
        .read_to_string(&mut content)
        .chain(ErrorKind::ReadFromTemplateFailed(template_path_string))?;
    let content = content.replace("$LIB_DIR", &libdir_escaped());
    create_and_write_file(path, &content)
}

fn create_and_write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).chain(ErrorKind::CreateDestinationDirectoryFailed(
            parent.display().to_string(),
        ))?;
    }
    let path_string = path.display().to_string();
    let mut f = File::create(path).chain(ErrorKind::CreateDestinationFailed(path_string.clone()))?;
    f.write_all(content.as_bytes())
        .chain(ErrorKind::WriteToDestinationFailed(path_string))
}

fn libdir_escaped() -> String {
    common::get_procon_lib_dir()
        .display()
        .to_string()
        .escape_default()
}
