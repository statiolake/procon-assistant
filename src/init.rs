use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::imp::common;
use crate::imp::config::ConfigFile;
use crate::imp::langs;
use crate::imp::langs::Lang;

const FILES: &[&str] = &[
    ".clang_complete",
    "compile_commands.json",
    ".vscode/c_cpp_properties.json",
    ".vscode/tasks.json",
    ".vscode/launch.json",
];

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); format!("failed to get config.")];
    [ProjectNameNotSpecified; (); format!("project name was not specified.")];
    [UnknownFileType; (lang: String); format!("unknown file type: {}", lang)];
    [CreateDestinationDirectoryFailed; (name: String); format!("creating directory `{}' failed.", name)];
    [CreateDestinationFailed; (name: String); format!("creating `{}' failed.", name)];
    [WriteToDestinationFailed; (name: String); format!("writing `{}' failed.", name)];
    [OpenTemplateFailed; (name: String); format!("template file for `{}' not found.", name)];
    [ReadFromTemplateFailed; (name: String); format!("reading from template `{}' failed.", name)];
    [OpeningEditorFailed; (); format!("failed to open editor.")];
}

struct CmdOpt {
    name: String,
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

        name.ok_or(Error::new(ErrorKind::ProjectNameNotSpecified()))
            .map(|name| CmdOpt { name, lang })
    }
}

struct Project {
    name: String,
    lang: &'static Lang,
}

pub fn main(args: Vec<String>) -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;

    // parse command line arguments
    let cmdopt = CmdOpt::parse(args)?;
    let project = validate_arguments(&config, cmdopt)?;

    let path_project = Path::new(&project.name);
    create_project_directory(&path_project)?;

    let path_src_file = Path::new(&project.lang.src_file_name);
    safe_generate(&project.lang, path_project, path_src_file)?;

    for file in FILES {
        let path = Path::new(file);
        safe_generate(&project.lang, path_project, path)?;
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
        common::open(&config, &[&path_open]).chain(ErrorKind::OpeningEditorFailed())?;
    }

    Ok(())
}

fn create_project_directory(path_project: &Path) -> Result<()> {
    fs::create_dir_all(path_project).chain(ErrorKind::CreateDestinationDirectoryFailed(
        path_project.display().to_string(),
    ))
}

fn safe_generate(lang: &Lang, path_project: &Path, path: &Path) -> Result<()> {
    if path_project.join(path).exists() {
        print_info!(true, "file {} already exists, skipping.", path.display());
        return Ok(());
    }

    generate(lang, path_project, path)?;
    print_generated!("{}", path.display());
    Ok(())
}

fn generate(lang: &Lang, path_project: &Path, path: &Path) -> Result<()> {
    let exe_dir = current_exe::current_exe().unwrap();
    let exe_dir = exe_dir.parent().unwrap();
    let path_template = exe_dir.join("template").join(path);
    let path_project = path_project.join(path);

    let path_template_string = path_template.display().to_string();
    print_info!(true, "loading template from `{}'", path_template_string);
    let mut template_file = File::open(path_template)
        .chain(ErrorKind::OpenTemplateFailed(path_template_string.clone()))?;

    let mut content = String::new();
    template_file
        .read_to_string(&mut content)
        .chain(ErrorKind::ReadFromTemplateFailed(path_template_string))?;

    let content = content.replace("$LIB_DIR", &libdir_escaped(&lang));
    let content = content.replace("$GDB_PATH", &gdbpath_escaped());
    create_and_write_file(&path_project, &content)
}

fn validate_arguments(config: &ConfigFile, cmdopt: CmdOpt) -> Result<Project> {
    let name = cmdopt.name;

    // generate source code
    let specified_lang = cmdopt.lang.unwrap_or(config.init_default_lang.clone());

    let lang = langs::FILETYPE_ALIAS
        .get(&*specified_lang)
        .ok_or(Error::new(ErrorKind::UnknownFileType(specified_lang)))?;

    let lang = langs::LANGS_MAP
        .get(lang)
        .expect(&format!("internal error: unknown file type {}", lang));

    Ok(Project { name, lang })
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

fn gdbpath_escaped() -> String {
    which::which("gdb")
        .map(|path| path.display().to_string().escape_default())
        .unwrap_or("dummy - could not find GDB in your system".into())
}

fn libdir_escaped(lang: &Lang) -> String {
    (lang.lib_dir_getter)()
        .display()
        .to_string()
        .escape_default()
}
