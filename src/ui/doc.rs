use crate::imp::langs;
use crate::ExitStatus;
use anyhow::{Context, Result};
use clap::Clap;

#[derive(Clap)]
#[clap(about = "Opens a document for the current project")]
pub struct Doc;

impl Doc {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_lang().context("failed to guess the current language")?;
        lang.open_docs().context("failed to open documents")?;

        Ok(ExitStatus::Success)
    }
}
