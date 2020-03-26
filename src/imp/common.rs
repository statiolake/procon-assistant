use crate::imp::config::ConfigFile;
use std::process::Command;

pub fn colorize() -> bool {
    use atty::Stream;
    atty::is(Stream::Stderr)
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to spawn command `{process_name}`")]
    SpawningCommandFailed {
        source: anyhow::Error,
        process_name: String,
    },
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
        .map_err(|e| Error::SpawningCommandFailed {
            source: e.into(),
            process_name: process_name.to_string(),
        })?;
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
