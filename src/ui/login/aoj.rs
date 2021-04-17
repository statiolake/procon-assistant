use super::LoginUi;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Logs in to Aoj")]
pub struct Aoj;

impl Aoj {
    pub fn run(self, quiet: bool) -> Result<()> {
        Aoj.authenticate(quiet).context("failed to login")
    }
}

impl LoginUi for Aoj {
    fn authenticate(&self, _quiet: bool) -> Result<()> {
        // TODO: implement
        Ok(())
    }
}
