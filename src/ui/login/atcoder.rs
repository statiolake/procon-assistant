use super::LoginUI;
use crate::eprintln_tagged;
use crate::imp::auth;
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Logs in to AtCoder")]
pub struct AtCoder;

impl AtCoder {
    pub fn run(self, quiet: bool) -> Result<()> {
        AtCoder.authenticate(quiet).context("failed to login")
    }
}

impl LoginUI for AtCoder {
    fn authenticate(&self, _quiet: bool) -> Result<()> {
        let (username, password) = auth::ask_account_info("AtCoder");
        eprintln_tagged!("Logging in": "to AtCoder");
        auth::atcoder::login(&username, &password)?;
        eprintln_tagged!("Finished": "fetching code; successfully saved");

        Ok(())
    }
}
