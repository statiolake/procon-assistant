use serde_derive::Deserialize;
use std::fs::File;
use which::which;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to get config")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("`config.json` is not found")]
    ConfigFileMissing { source: anyhow::Error },

    #[error("failed to parse `config.json`")]
    ErrorInConfigFile { source: anyhow::Error },
}

#[derive(Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub init: Init,
    #[serde(default)]
    pub addcase: Addcase,
    #[serde(default)]
    pub run: Run,
    #[serde(default)]
    pub languages: Languages,
}

#[derive(Deserialize)]
pub struct General {
    #[serde(default = "General::default_editor_command")]
    pub editor_command: Vec<String>,
    #[serde(default = "General::default_wait_for_editor_finish")]
    pub wait_for_editor_finish: bool,
}

#[derive(Deserialize)]
pub struct Init {
    #[serde(default = "Init::default_auto_open")]
    pub auto_open: bool,
    #[serde(default = "Init::default_open_target")]
    pub open_target: OpenTarget,
    #[serde(default = "Init::default_default_lang")]
    pub default_lang: String,
}

#[derive(Deserialize)]
pub enum OpenTarget {
    #[serde(rename = "file")]
    Files,
    #[serde(rename = "directory")]
    Directory,
}

#[derive(Deserialize)]
pub struct Addcase {
    #[serde(default = "Addcase::default_give_argument_once")]
    pub give_argument_once: bool,
    #[serde(default = "Addcase::default_editor_command")]
    pub editor_command: Vec<String>,
    #[serde(default = "Addcase::default_wait_for_editor_finish")]
    pub wait_for_editor_finish: bool,
}

#[derive(Deserialize)]
pub struct Run {
    #[serde(default = "Run::default_timeout_milliseconds")]
    pub timeout_milliseconds: u64,
    #[serde(default = "Run::default_eps_for_float")]
    pub eps_for_float: f64,
}

#[derive(Deserialize, Default)]
pub struct Languages {
    #[serde(default)]
    pub rust: Rust,
}

#[derive(Deserialize)]
pub struct Rust {
    #[serde(default = "Rust::default_project_template")]
    pub project_template: RustProjectTemplate,
    #[serde(default = "Rust::default_needs_pre_compile")]
    pub needs_pre_compile: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum RustProjectTemplate {
    #[serde(rename = "git")]
    Git { repository: String, branch: String },

    #[serde(rename = "local")]
    Local { path: String },
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = current_exe::current_exe()
            .unwrap()
            .with_file_name("config.json");

        let file = match File::open(&config_path) {
            Ok(f) => f,
            Err(_) => return Ok(ConfigFile::default()),
        };

        serde_json::from_reader(file)
            .map_err(|e| Error(ErrorKind::ErrorInConfigFile { source: e.into() }))
    }
}

impl Default for General {
    fn default() -> Self {
        General {
            editor_command: General::default_editor_command(),
            wait_for_editor_finish: General::default_wait_for_editor_finish(),
        }
    }
}

impl General {
    pub fn default_editor_command() -> Vec<String> {
        if let Ok(vscode) = which("code") {
            vec![vscode.display().to_string(), "%PATHS%".to_string()]
        } else if cfg!(windows) {
            vec!["notepad.exe".to_string(), "%PATHS%".to_string()]
        } else {
            vec!["vi".to_string(), "%PATHS%".to_string()]
        }
    }

    // To allow codes written a bit verbosely to improve readability
    #[allow(clippy::needless_bool, clippy::if_same_then_else)]
    pub fn default_wait_for_editor_finish() -> bool {
        if which("code").is_ok() {
            // vscode
            false
        } else if cfg!(windows) {
            // notepad
            false
        } else {
            // vi
            true
        }
    }
}

impl Default for Init {
    fn default() -> Self {
        Init {
            auto_open: Init::default_auto_open(),
            open_target: Init::default_open_target(),
            default_lang: Init::default_default_lang(),
        }
    }
}

impl Init {
    pub fn default_auto_open() -> bool {
        true
    }

    pub fn default_open_target() -> OpenTarget {
        OpenTarget::Files
    }

    pub fn default_default_lang() -> String {
        "cpp".to_string()
    }
}

impl Default for Addcase {
    fn default() -> Self {
        Addcase {
            give_argument_once: Addcase::default_give_argument_once(),
            editor_command: Addcase::default_editor_command(),
            wait_for_editor_finish: Addcase::default_wait_for_editor_finish(),
        }
    }
}

impl Addcase {
    pub fn default_give_argument_once() -> bool {
        false
    }

    pub fn default_editor_command() -> Vec<String> {
        General::default_editor_command()
    }

    pub fn default_wait_for_editor_finish() -> bool {
        General::default_wait_for_editor_finish()
    }
}

impl Default for Run {
    fn default() -> Self {
        Run {
            timeout_milliseconds: Run::default_timeout_milliseconds(),
            eps_for_float: Run::default_eps_for_float(),
        }
    }
}

impl Run {
    pub fn default_timeout_milliseconds() -> u64 {
        3000
    }

    pub fn default_eps_for_float() -> f64 {
        1e-8
    }
}

impl Default for Rust {
    fn default() -> Self {
        Rust {
            project_template: Rust::default_project_template(),
            needs_pre_compile: Rust::default_needs_pre_compile(),
        }
    }
}

impl Rust {
    pub fn default_project_template() -> RustProjectTemplate {
        RustProjectTemplate::Git {
            repository: "https://github.com/rust-lang-ja/atcoder-rust-base".to_string(),
            branch: "ja".to_string(),
        }
    }

    pub fn default_needs_pre_compile() -> bool {
        true
    }
}
