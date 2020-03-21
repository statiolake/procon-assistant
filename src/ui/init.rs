use crate::imp::common;
use crate::imp::config::ConfigFile;
use crate::imp::langs;
use crate::imp::langs::Lang;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const FILES: &[&str] = &[
    "compile_commands.json",
    ".vscode/c_cpp_properties.json",
    ".vscode/tasks.json",
    ".vscode/launch.json",
];

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get config.")]
    GettingConfigFailed { source: anyhow::Error },

    #[error("unknown file type: {lang}")]
    UnknownFileType { lang: String },

    #[error("creating directory `{name}` failed.")]
    CreateDestinationDirectoryFailed { source: anyhow::Error, name: String },

    #[error("creating `{name}` failed.")]
    CreateDestinationFailed { source: anyhow::Error, name: String },

    #[error("writing `{name}` failed.")]
    WriteToDestinationFailed { source: anyhow::Error, name: String },

    #[error("template file for `{name}` not found.")]
    OpenTemplateFailed { source: anyhow::Error, name: String },

    #[error("template variable substitution failed.")]
    TemplateVariableSubstitutionFailed { source: anyhow::Error },

    #[error("reading from template `{name}` failed.")]
    ReadFromTemplateFailed { source: anyhow::Error, name: String },

    #[error("failed to open editor.")]
    OpeningEditorFailed { source: anyhow::Error },
}

struct CmdOpt {
    name: Option<String>,
    lang: Option<String>,
}

impl CmdOpt {
    pub fn parse(args: Vec<String>) -> Result<CmdOpt> {
        let mut args = args.into_iter();

        let mut name = None;
        let mut lang = None;

        while let Some(arg) = args.next() {
            match &*arg {
                "-t" | "--type" => lang = args.next(),
                _ => name = Some(arg),
            }
        }

        Ok(CmdOpt { name, lang })
    }
}

struct Project {
    name: String,
    lang: &'static Lang,
}

pub fn main(quiet: bool, args: Vec<String>) -> Result<()> {
    let config: ConfigFile =
        ConfigFile::get_config().map_err(|e| Error::GettingConfigFailed { source: e.into() })?;

    // parse command line arguments
    let cmdopt = CmdOpt::parse(args)?;
    let project = validate_arguments(&config, cmdopt)?;

    let path_project = Path::new(&project.name);
    create_project_directory(&path_project)?;

    let path_src_file = Path::new(&project.lang.src_file_name);
    safe_generate(quiet, &project.lang, path_project, path_src_file)?;

    for file in FILES {
        let path = Path::new(file);
        safe_generate(quiet, &project.lang, path_project, path)?;
    }

    if config.init_auto_open {
        let path_open = if config.init_open_directory_instead_of_specific_file {
            path_project.display().to_string()
        } else {
            path_project
                .join(&project.lang.src_file_name)
                .display()
                .to_string()
        };
        common::open(&config, false, &[&path_open])
            .map_err(|e| Error::OpeningEditorFailed { source: e.into() })?;
    }

    Ok(())
}

fn create_project_directory(path_project: &Path) -> Result<()> {
    fs::create_dir_all(path_project).map_err(|e| Error::CreateDestinationDirectoryFailed {
        source: e.into(),
        name: path_project.display().to_string(),
    })
}

fn safe_generate(quiet: bool, lang: &Lang, path_project: &Path, path: &Path) -> Result<()> {
    if path_project.join(path).exists() {
        print_info!(!quiet, "file {} already exists, skipping.", path.display());
        return Ok(());
    }

    generate(quiet, lang, path_project, path)?;
    print_generated!("{}", path.display());
    Ok(())
}

fn generate(quiet: bool, lang: &Lang, path_project_root: &Path, path: &Path) -> Result<()> {
    let exe_dir = current_exe::current_exe().unwrap();
    let exe_dir = exe_dir.parent().unwrap();
    let path_template = exe_dir.join("template").join(path);
    let path_project = path_project_root.join(path);

    let path_template_string = path_template.display().to_string();
    print_info!(!quiet, "loading template from `{}'", path_template_string);
    let mut template_file = File::open(path_template).map_err(|e| Error::OpenTemplateFailed {
        source: e.into(),
        name: path_template_string.clone(),
    })?;

    let mut content = String::new();
    template_file
        .read_to_string(&mut content)
        .map_err(|e| Error::ReadFromTemplateFailed {
            source: e.into(),
            name: path_template_string,
        })?;

    let content = content.replace("$LIB_DIR", &libdir_escaped(&lang));
    let content = content.replace("$GDB_PATH", &gdbpath_escaped());

    let abs_path_project_root = to_absolute::to_absolute_from_current_dir(path_project_root)
        .map_err(|e| Error::TemplateVariableSubstitutionFailed { source: e.into() })?;
    let content = content.replace("$PROJECT_PATH", &escape_path(abs_path_project_root));

    create_and_write_file(&path_project, &content)
}

fn validate_arguments(config: &ConfigFile, cmdopt: CmdOpt) -> Result<Project> {
    let name = cmdopt.name.unwrap_or_else(|| ".".into());

    // generate source code
    let specified_lang = cmdopt
        .lang
        .unwrap_or_else(|| config.init_default_lang.clone());

    let lang =
        langs::FILETYPE_ALIAS
            .get(&*specified_lang)
            .ok_or_else(|| Error::UnknownFileType {
                lang: specified_lang,
            })?;

    let lang = langs::LANGS_MAP
        .get(lang)
        .unwrap_or_else(|| panic!("internal error: unknown file type {}", lang));

    Ok(Project { name, lang })
}

fn create_and_write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::CreateDestinationDirectoryFailed {
            source: e.into(),
            name: parent.display().to_string(),
        })?;
    }
    let path_string = path.display().to_string();
    let mut f = File::create(path).map_err(|e| Error::CreateDestinationFailed {
        source: e.into(),
        name: path_string.clone(),
    })?;
    f.write_all(content.as_bytes())
        .map_err(|e| Error::WriteToDestinationFailed {
            source: e.into(),
            name: path_string,
        })
}

fn gdbpath_escaped() -> String {
    which::which("gdb")
        .map(escape_path)
        .unwrap_or_else(|_| "dummy - could not find GDB in your system".into())
}

fn libdir_escaped(lang: &Lang) -> String {
    let libdir = (lang.lib_dir_getter)();
    escape_path(libdir)
}

fn escape_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .display()
        .to_string()
        .escape_default()
        .to_string()
}
