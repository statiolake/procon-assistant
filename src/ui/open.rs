use crate::imp::config::OpenTarget;
use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::imp::process;
use crate::ExitStatus;
use anyhow::{Context, Result};
use itertools::Itertools;
use scopeguard::defer;
use std::env;
use std::path::Path;

#[derive(clap::Clap)]
#[clap(about = "Open generated files in a directory")]
pub struct Open {
    #[clap(
        default_value = ".",
        help = "The name of directory; if `.`, open current directory"
    )]
    dirname: String,
}

impl Open {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        open(Path::new(&self.dirname))?;
        Ok(ExitStatus::Success)
    }
}

pub fn open(path: &Path) -> Result<()> {
    let cwd = env::current_dir().expect("critical error: failed to get current directory");
    env::set_current_dir(path)
        .with_context(|| format!("failed to open directory `{}`", path.display()))?;
    defer! {
        env::set_current_dir(&cwd)
            .expect("critical error: failed to go back to the original working directory");
    }

    let lang = langs::guess_lang().context("failed to guess the language of the project")?;
    let to_open = lang.to_open(path);
    let (to_open, cwd) = match CONFIG.open.open_target {
        OpenTarget::Directory => (vec![to_open.directory], None),
        OpenTarget::Files => {
            let cwd = Path::new(&to_open.directory);
            let files = to_open
                .files
                .into_iter()
                .map(|file| {
                    file.strip_prefix(cwd)
                        .expect("critical error: files are not under the base directory")
                        .to_path_buf()
                })
                .collect_vec();
            (files, Some(cwd))
        }
    };
    process::open_general(&to_open, cwd).context("failed to open the editor")?;

    Ok(())
}
