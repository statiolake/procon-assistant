use crate::eprintln_info;
use crate::imp::langs;
use crate::ExitStatus;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(clap::Parser)]
#[clap(help = "Opens a document for the current project")]
pub struct Doc;

impl Doc {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang =
            langs::guess_lang().context("failed to guess the language of the current project")?;
        if !quiet {
            eprintln_info!("guessed language: {}", lang.get_lang_name());
        }
        lang.open_docs().context("failed to open documents")?;

        Ok(ExitStatus::Success)
    }
}
