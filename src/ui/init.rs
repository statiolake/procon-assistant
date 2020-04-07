use crate::eprintln_progress;
use crate::imp::common;
use crate::imp::config::OpenTarget;
use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::ExitStatus;
use anyhow::anyhow;
use anyhow::{Context, Result};
use std::path::Path;

#[derive(clap::Clap)]
#[clap(about = "Generates files in a directory")]
pub struct Init {
    #[clap(
        default_value = ".",
        help = "The name of directory; if `.`, files will be generated in the current directory"
    )]
    dirname: String,

    #[clap(short, long, help = "The lang to init")]
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

        let to_open = progress
            .handle
            .join()
            .map_err(|_| anyhow!("init thread panicked"))?
            .context("init failed")?;

        if CONFIG.init.auto_open {
            let to_open = match CONFIG.init.open_target {
                OpenTarget::Directory => vec![to_open.directory],
                OpenTarget::Files => to_open.files,
            };

            common::open_general(&to_open).context("failed to open the editor")?;
        }

        Ok(ExitStatus::Success)
    }
}
