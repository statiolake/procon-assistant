use crate::eprintln_debug;
use crate::ui::CONFIG;
use itertools::Itertools;
use std::path::Path;
use std::process::{Command, Stdio};

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

pub fn open_addcase<P: AsRef<Path>>(names: &[P]) -> Result<()> {
    let names = names
        .iter()
        .map(|p| p.as_ref().display().to_string())
        .collect_vec();
    let names = names.iter().map(String::as_str).collect_vec();
    let (process_name, cmds) = open_addcase_cmds(&names)?;
    spawn_editor(process_name, cmds)
}

pub fn open_general<P: AsRef<Path>>(names: &[P]) -> Result<()> {
    let names = names
        .iter()
        .map(|p| p.as_ref().display().to_string())
        .inspect(|p| eprintln_debug!("Opening `{}`", p))
        .collect_vec();
    let names = names.iter().map(String::as_str).collect_vec();
    let (process_name, cmds) = open_general_cmds(&names)?;
    spawn_editor(process_name, cmds)
}

fn spawn_editor(process_name: &str, cmds: Vec<Command>) -> Result<()> {
    for mut cmd in cmds {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let res = if CONFIG.general.wait_for_editor_finish {
            cmd.spawn().and_then(|mut child| child.wait()).map(drop)
        } else {
            cmd.spawn().map(drop)
        };

        if let Err(e) = res {
            return Err(Error::SpawningCommandFailed {
                source: e.into(),
                process_name: process_name.to_string(),
            });
        }
    }

    Ok(())
}

fn open_addcase_cmds(names: &[&str]) -> Result<(&'static str, Vec<Command>)> {
    let mut editor_command = CONFIG.addcase.editor_command.iter().map(String::as_str);
    let process_name = editor_command.next().unwrap_or("");
    let editor_command = editor_command.collect_vec();

    let mut commands = Vec::new();
    if CONFIG.addcase.give_argument_once {
        let command = make_editor_command(process_name, &editor_command, names);
        commands.push(command);
    } else {
        for name in names {
            let command = make_editor_command(process_name, &editor_command.clone(), &[name]);
            commands.push(command);
        }
    }

    Ok((process_name, commands))
}

fn open_general_cmds(names: &[&str]) -> Result<(&'static str, Vec<Command>)> {
    let mut editor_command = CONFIG.general.editor_command.iter().map(|x| x as &str);
    let process_name = editor_command.next().unwrap_or("");
    let editor_command = editor_command.collect_vec();

    let commands = vec![make_editor_command(process_name, &editor_command, names)];

    Ok((process_name, commands))
}

fn make_editor_command(process_name: &str, arguments: &[&str], file_paths: &[&str]) -> Command {
    let arguments = arguments.iter().flat_map(|arg| match *arg {
        "%PATHS%" => file_paths.iter().map(ToString::to_string).collect_vec(),
        other => vec![other.to_string()],
    });
    let mut cmd = Command::new(process_name);
    cmd.args(arguments);

    cmd
}
