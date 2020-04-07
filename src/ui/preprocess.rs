use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::imp::langs::Preprocessed;
use crate::ExitStatus;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Preprocesses current source file and prepares to submit")]
pub struct Preprocess;

impl Preprocess {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let lang =
            langs::guess_lang().context("failed to guess the language used in this project")?;
        let source = lang.get_source().context("failed to read the sorce file")?;
        let Preprocessed(pped) = lang
            .preprocess(&source, CONFIG.clip.minify)
            .context("failed to preprocess the source")?;

        println!("{}", pped);

        Ok(ExitStatus::Success)
    }
}
