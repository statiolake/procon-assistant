use serde_derive::Deserialize;
use std::fs::File;

pub const TIMEOUT_MILLISECOND: u64 = 3000;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("failed to get config")]
pub struct Error(ErrorKind);

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("`config.json` is not found")]
    ConfigFileMissing { source: anyhow::Error },

    #[error("failed to parse `config.json`")]
    ErrorInConfigFile { source: anyhow::Error },
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub editor_command: Vec<String>,
    // for terminal editor like vim
    pub is_terminal_editor: bool,
    pub init_auto_open: bool,
    pub init_open_directory_instead_of_specific_file: bool,
    pub init_default_lang: String,
    pub addcase_give_argument_once: bool,
    pub addcase_editor_command: Vec<String>,
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = current_exe::current_exe()
            .unwrap()
            .with_file_name("config.json");
        File::open(&config_path)
            .map_err(|e| Error(ErrorKind::ConfigFileMissing { source: e.into() }))
            .and_then(|f| {
                serde_json::from_reader(f)
                    .map_err(|e| Error(ErrorKind::ErrorInConfigFile { source: e.into() }))
            })
    }
}
