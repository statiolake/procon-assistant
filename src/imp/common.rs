use dirs;
use isatty;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use imp::config::ConfigFile;

pub fn colorize() -> bool {
    isatty::stdout_isatty()
}

define_error!();
define_error_kind! {
    [SpawningCommandFailed; (command_name: String); format!("failed to spawn command `{}'", command_name)];
    [CreatingFailed; (file_name: String); format!("failed to create file `{}'", file_name)];
    [WritingFailed; (file_name: String); format!("failed to write to file `{}'", file_name)];
}

pub fn open(config: &ConfigFile, names: &[&str]) -> Result<()> {
    let mut commands = Vec::new();
    if config.addcase_give_argument_once {
        let mut command = Command::new(&config.editor);
        command.args(names);
        commands.push(command);
    } else {
        for name in names {
            let mut command = Command::new(&config.editor);
            command.arg(name);
            commands.push(command);
        }
    }

    for mut command in commands {
        if config.addcase_wait_editor_finish {
            use std::process::Stdio;
            command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .map(|_| ())
        } else {
            command.spawn().map(|_| ())
        }
        .chain(ErrorKind::SpawningCommandFailed(config.editor.to_string()))?;
    }
    Ok(())
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
    dirs::home_dir().expect("critical error: failed to get home_dir")
}
