use isatty;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use config;
use Error;
use Result;

pub fn colorize() -> bool {
    isatty::stdout_isatty()
}

pub fn open(name: &str) -> Result<()> {
    Command::new("open")
        .arg(name)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            Error::with_cause(
                format!("opening {}", name),
                "failed to spawn open command.",
                box e,
            )
        })
}

pub fn ensure_to_create_file(name: &str, text: &str) -> Result<()> {
    let mut f = File::create(name)
        .map_err(|e| Error::with_cause(format!("creating {}", name), "failed", box e))?;

    if text != "" {
        f.write_all(text.as_bytes())
            .map_err(|e| Error::with_cause(format!("writing into {}", name), "failed", box e))?;
    }

    Ok(())
}

pub fn get_home_path() -> Result<PathBuf> {
    env::home_dir().ok_or(Error::new(
        "generating compile option",
        "can't get home directory's path.",
    ))
}

pub fn get_procon_lib_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_path()?;
    home_dir.push(config::src_support::cpp::PROCON_LIB_DIR);
    Ok(home_dir)
}
