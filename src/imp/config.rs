use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use which::which;

lazy_static! {
    pub static ref CONFIG: ConfigFile = ConfigFile::get_config().expect(concat!(
        "critical error: failed to get the config;",
        " Make sure you get config once before using CONFIG",
        " and handle errors earlier"
    ));
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
    pub clip: Clip,
    #[serde(default)]
    pub langs: Langs,
    #[serde(default)]
    pub doc: Doc,
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

#[derive(Deserialize)]
pub struct Clip {
    #[serde(default = "Clip::default_minify")]
    pub minify: MinifyMode,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MinifyMode {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "template_only")]
    TemplateOnly,
    #[serde(rename = "all")]
    All,
}

#[derive(Deserialize, Default)]
pub struct Langs {
    #[serde(default)]
    pub cpp: Cpp,
    #[serde(default)]
    pub rust: Rust,
}

#[derive(Deserialize)]
pub struct Cpp {
    #[serde(default = "Cpp::default_template_dir")]
    pub template_dir: PathBuf,
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
    Local { path: PathBuf },
}

#[derive(Deserialize, Default)]
pub struct Doc {
    #[serde(default)]
    pub browser: Option<Vec<String>>,
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = config_dir().join("config.json");

        let file = match File::open(&config_path) {
            Ok(f) => f,
            Err(_) => return Ok(ConfigFile::default()),
        };

        serde_json::from_reader(file).context("error in config file")
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

impl Default for Clip {
    fn default() -> Self {
        Clip {
            minify: Clip::default_minify(),
        }
    }
}

impl Clip {
    pub fn default_minify() -> MinifyMode {
        MinifyMode::TemplateOnly
    }
}

impl Default for Cpp {
    fn default() -> Self {
        Cpp {
            template_dir: Cpp::default_template_dir(),
        }
    }
}

impl Cpp {
    pub fn default_template_dir() -> PathBuf {
        config_dir().join("template").join("cpp")
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

pub fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("procon-assistant")
}
