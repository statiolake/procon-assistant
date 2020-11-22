use crate::eprintln_debug;
use crate::imp::config::OpenTarget;
use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::imp::process;
use crate::ExitStatus;
use anyhow::{Context, Result};
use itertools::Itertools;
use scopeguard::defer;
use std::cell::RefCell;
use std::env;
use std::path::Path;

#[derive(clap::Clap)]
#[clap(about = "Open generated files in a directory")]
pub struct Open {
    #[clap(
        default_value = ".",
        about = "The name of directory; if `.`, open current directory"
    )]
    dirname: String,
}

impl Open {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        // defer must be outer function of the below `if` statement (if this is
        // defined in `if` statement, this function is called when the if
        // statement ends, which is too early from ending the entire function).
        let original_dir = RefCell::new(None);
        defer! {
            if let Some(original_dir) = &*original_dir.borrow() {
                // restore original directory later
                env::set_current_dir(original_dir)
                    .expect("critical error: failed to restore original directory");
            }
        }

        // if dirname is not current dir (`.`), fix the current directory
        if self.dirname != "." {
            eprintln_debug!(
                "dirname is not `.` but {}; creating project directory",
                self.dirname
            );

            // save current directory and fix current directory
            *original_dir.borrow_mut() =
                Some(env::current_dir().expect("critical error: failed to get current directory"));
            env::set_current_dir(&self.dirname)
                .expect("critical error: failed to set current directory to the project directory");
        }
        open()?;

        Ok(ExitStatus::Success)
    }
}

pub fn open() -> Result<()> {
    let lang = langs::guess_lang().context("failed to guess the language of the project")?;
    let to_open = lang.to_open().context("failed to get files to open")?;
    let (to_open, cwd) = match CONFIG.open.open_target {
        OpenTarget::Directory => (vec![to_open.directory], None),
        OpenTarget::Files => {
            eprintln_debug!(
                "directory: {}, files: {:?}",
                to_open.directory.display(),
                &to_open.files
            );

            let cwd = Path::new(&to_open.directory);

            let files = to_open
                .files
                .into_iter()
                .map(|file| {
                    // strip the prefix, but only when the base directory is non-trivial.
                    if cwd.display().to_string() != "." {
                        file.strip_prefix(cwd)
                            .expect("critical error: files are not under the base directory")
                            .to_path_buf()
                    } else {
                        file
                    }
                })
                .collect_vec();
            (files, Some(cwd))
        }
    };
    process::open_general(&to_open, cwd).context("failed to open the editor")?;

    Ok(())
}
