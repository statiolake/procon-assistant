use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use crate::imp::config::ConfigFile;

pub fn colorize() -> bool {
    use atty::Stream;
    atty::is(Stream::Stderr)
}

define_error!();
define_error_kind! {
    [SpawningCommandFailed; (command_name: String); format!("failed to spawn command `{}'", command_name)];
    [CreatingFailed; (file_name: String); format!("failed to create file `{}'", file_name)];
    [WritingFailed; (file_name: String); format!("failed to write to file `{}'", file_name)];
}

pub fn open(config: &ConfigFile, addcase: bool, names: &[&str]) -> Result<()> {
    let (process_name, commands) = if addcase {
        open_addcase(config, names)
    } else {
        open_general(config, names)
    }?;

    for mut command in commands {
        if config.is_terminal_editor {
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
        .chain(ErrorKind::SpawningCommandFailed(process_name.to_string()))?;
    }
    Ok(())
}

fn open_addcase<'a, 'b, 'c>(
    config: &'a ConfigFile,
    names: &'b [&'c str],
) -> Result<(&'a str, Vec<Command>)> {
    let mut editor_command = config.addcase_editor_command.iter().map(|x| x as &str);
    let process_name = editor_command.next().unwrap_or("");

    // make rustc re-infer lifetime of iterator item: &'? str without this, process_name is of type
    // &'a str (since it is return value) and so editor_command is of type Iterator<Item = &'a
    // str>.  however make_editor_command requires this iterator and iterator based on `names` must
    // be the iterator generating same type, so that causes lifetime error.
    let editor_command = editor_command.map(|x| &*x);

    let mut commands = Vec::new();
    if config.addcase_give_argument_once {
        let command = make_editor_command(process_name, editor_command, names.iter().cloned());
        commands.push(command);
    } else {
        for name in names {
            let command = make_editor_command(
                process_name,
                editor_command.clone(),
                Some(name).into_iter().cloned(),
            );
            commands.push(command);
        }
    }

    Ok((process_name, commands))
}

fn open_general<'a, 'b>(
    config: &'a ConfigFile,
    names: &'b [&str],
) -> Result<(&'a str, Vec<Command>)> {
    let mut editor_command = config.editor_command.iter().map(|x| x as &str);
    let process_name = editor_command.next().unwrap_or("");

    // make rustc re-infer lifetime of iterator
    let editor_command = editor_command.map(|x| &*x);
    let commands = vec![make_editor_command(
        process_name,
        editor_command,
        names.iter().cloned(),
    )];

    Ok((process_name, commands))
}

fn make_editor_command<'a, 'b>(
    process_name: &'a str,
    arguments: impl Iterator<Item = &'b str>,
    file_paths: impl Iterator<Item = &'b str> + Clone,
) -> Command {
    let arguments = arguments.flat_map(|arg| match arg {
        "%PATHS%" => Box::new(file_paths.clone()) as Box<dyn Iterator<Item = &'b str>>,
        _ => Box::new(Some(arg).into_iter()) as Box<dyn Iterator<Item = &'b str>>,
    });
    let mut cmd = Command::new(process_name);
    cmd.args(arguments);

    cmd
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
