use crate::eprintln_progress;
use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::ui::open;
use crate::ExitStatus;
use anyhow::anyhow;
use anyhow::{Context, Result};
use std::path::Path;

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
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let specified_lang = self.lang.as_ref().unwrap_or(&CONFIG.init.default_lang);

        let lang = langs::get_from_alias(specified_lang).context("failed to get the language")?;
        let path_project = Path::new(&self.dirname);

        // initialize the project asynchronously and get progress
        let progress = lang.init_async(path_project);
        while let Ok(msg) = progress.recver.recv() {
            eprintln_progress!("{}", msg);
        }

        progress
            .handle
            .join()
            .map_err(|_| anyhow!("init thread panicked"))?
            .context("init failed")?;

        if CONFIG.init.auto_open {
            open::open(&path_project).context("failed to open the generated project")?;
        }

        Ok(ExitStatus::Success)
    }
}
