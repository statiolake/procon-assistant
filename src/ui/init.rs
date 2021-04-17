use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::ui::open;
use crate::ExitStatus;
use crate::{eprintln_debug, eprintln_progress};
use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use scopeguard::defer;
use std::cell::RefCell;
use std::path::MAIN_SEPARATOR;
use std::{env, fs};

#[derive(clap::Clap)]
#[clap(about = "Generates files in a directory")]
pub struct Init {
    #[clap(
        default_value = ".",
        about = "The name of directory; if `.`, files will be generated in the current directory"
    )]
    dirname: String,

    #[clap(short, long, about = "The lang to init")]
    lang: Option<String>,
}

impl Init {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let specified_lang = self.lang.as_ref().unwrap_or(&CONFIG.init.default_lang);

        let lang = langs::get_from_alias(specified_lang).context("failed to get the language")?;

        // if dirname contains path separator, this is an error.
        if self.dirname.contains(MAIN_SEPARATOR) {
            bail!("dirname must not contain path separator: {}", self.dirname);
        }

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

            // create directory
            fs::create_dir_all(&self.dirname).context("failed to create project directory")?;

            // save current directory and fix current directory
            *original_dir.borrow_mut() =
                Some(env::current_dir().expect("critical error: failed to get current directory"));
            env::set_current_dir(&self.dirname)
                .expect("critical error: failed to set current directory to the project directory");
        }

        // initialize the project asynchronously and get progress
        let progress = lang.init_async();
        while let Ok(msg) = progress.recver.recv() {
            eprintln_progress!("{}", msg);
        }

        progress
            .handle
            .join()
            .map_err(|_| anyhow!("init thread panicked"))?
            .context("init failed")?;

        if CONFIG.init.auto_open {
            open::open(quiet).context("failed to open the generated project")?;
        }

        Ok(ExitStatus::Success)
    }
}
