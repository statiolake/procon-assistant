use isatty;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use imp::config;

pub fn colorize() -> bool {
    isatty::stdout_isatty()
}

define_error!();
define_error_kind! {
    [SpawningCommandFailed; (command_name: String); format!("failed to spawn command `{}'", command_name)];
    [CreatingFailed; (file_name: String); format!("failed to create file `{}'", file_name)];
    [WritingFailed; (file_name: String); format!("failed to write to file `{}'", file_name)];
}

pub fn open(editor: &str, name: &str) -> Result<()> {
    Command::new(editor)
        .arg(name)
        .spawn()
        .map(|_| ())
        .chain(ErrorKind::SpawningCommandFailed(editor.to_string()))
}

pub fn ensure_to_create_file(name: &str, text: &[u8]) -> Result<()> {
    let mut f = File::create(name).chain(ErrorKind::CreatingFailed(name.to_string()))?;

    if !text.is_empty() {
        f.write_all(text)
            .chain(ErrorKind::WritingFailed(name.to_string()))?;
    }

    Ok(())
}

pub fn get_home_path() -> PathBuf {
    env::home_dir().expect("critical error: failed to get home_dir")
}

pub fn get_procon_lib_dir() -> PathBuf {
    let mut home_dir = get_home_path();
    home_dir.push(config::src_support::cpp::PROCON_LIB_DIR);
    home_dir
}
