use crate::imp::langs;
use crate::imp::process;
use crate::ExitStatus;
use anyhow::{Context, Result};
use clap::Clap;

#[derive(Clap)]
#[clap(about = "Opens a document for the current project")]
pub struct Doc;

impl Doc {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_lang().context("failed to guess the current language")?;
        let doc_urls = lang.doc_urls().context("failed to get the document urls")?;
        for doc_url in doc_urls {
            process::open_browser(&doc_url)
                .with_context(|| format!("failed to open `{}`", doc_url))?;
        }

        Ok(ExitStatus::Success)
    }
}
