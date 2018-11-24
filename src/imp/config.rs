use serde_derive::Deserialize;

use std::env;
use std::fs::File;

use crate::tags::SPACER;

pub const TIMEOUT_MILLISECOND: i64 = 3000;

define_error!();
define_error_kind! {
    [ConfigFileMissing; (); format!(concat!(
        "`config.json' is missing.\n",
        "{}please check that file is placed at the same directory where this binary is placed."
    ), SPACER)];
    [ErrorInConfigFile; (); format!(concat!(
        "failed to parse config.json.\n",
        "{}maybe that file has syntax error, unknown options, or mismatched types."
    ), SPACER)];
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub editor: String,
    // for terminal editor like vim
    pub is_terminal_editor: bool,
    pub init_auto_open: bool,
    pub init_open_directory_instead_of_specific_file: bool,
    pub init_default_file_type: String,
    pub addcase_give_argument_once: bool,
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = env::current_exe().unwrap().with_file_name("config.json");
        File::open(&config_path)
            .chain(ErrorKind::ConfigFileMissing())
            .and_then(|f| serde_json::from_reader(f).chain(ErrorKind::ErrorInConfigFile()))
    }
}
