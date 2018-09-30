use imp::config::ConfigFile;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use imp::common;
use imp::langs;

const FILES: &[&str] = &[
    ".clang_complete",
    ".vscode/c_cpp_properties.json",
    ".vscode/tasks.json",
    ".vscode/launch.json",
];

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); format!("failed to get config.")];
    [UnknownFileType; (file_type: String); format!("unknown file type: {}", file_type)];
    [CreateDestinationDirectoryFailed; (name: String); format!("creating directory `{}' failed.", name)];
    [CreateDestinationFailed; (name: String); format!("creating `{}' failed.", name)];
    [WriteToDestinationFailed; (name: String); format!("writing `{}' failed.", name)];
    [OpenTemplateFailed; (name: String); format!("template file for `{}' not found.", name)];
    [ReadFromTemplateFailed; (name: String); format!("reading from template `{}' failed.", name)];
    [OpeningEditorFailed; (); format!("failed to open editor.")];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;

    // generate source code
    let specified_file_type = args
        .into_iter()
        .next()
        .unwrap_or(config.init_default_file_type.clone());
    let file_type = langs::FILETYPE_ALIAS
        .get(&*specified_file_type)
        .ok_or(Error::new(ErrorKind::UnknownFileType(specified_file_type)))?;
    safe_generate(Path::new(
        langs::LANGS_MAP
            .get(file_type)
            .expect(&format!("internal error: unknown file type {}", file_type))
            .src_file_name,
    ))?;

    for file in FILES {
        let path = Path::new(file);
        safe_generate(path)?;
    }

    if config.init_auto_open {
        match config.init_open_directory_instead_of_specific_file {
            true => common::open(&config, &["."]).chain(ErrorKind::OpeningEditorFailed())?,
            false => common::open(&config, &["main.cpp"]).chain(ErrorKind::OpeningEditorFailed())?,
        }
    }

    Ok(())
}

fn safe_generate(path: &Path) -> Result<()> {
    if path.exists() {
        print_info!(true, "file {} already exists, skipping.", path.display());
        return Ok(());
    }

    generate(path)?;
    print_generated!("{}", path.display());
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
    let mut f =
        File::create(path).chain(ErrorKind::CreateDestinationFailed(path_string.clone()))?;
    f.write_all(content.as_bytes())
        .chain(ErrorKind::WriteToDestinationFailed(path_string))
}

fn libdir_escaped() -> String {
    common::get_procon_lib_dir()
        .display()
        .to_string()
        .escape_default()
}
