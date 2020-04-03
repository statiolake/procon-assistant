use super::LoginUI;
use crate::eprintln_tagged;
use crate::imp::auth;

#[derive(clap::Clap)]
#[clap(about = "Logs in to AtCoder")]
pub struct AtCoder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to login")]
    LoginFailed { source: anyhow::Error },
}

impl AtCoder {
    pub fn run(self, quiet: bool) -> Result<()> {
        AtCoder
            .authenticate(quiet)
            .map_err(|e| Error::LoginFailed { source: e.into() })
    }
}

impl LoginUI for AtCoder {
    fn authenticate(&self, quiet: bool) -> anyhow::Result<()> {
        let (username, password) = auth::ask_account_info("AtCoder");
        eprintln_tagged!("Logging in": "to AtCoder");
        auth::atcoder::login(quiet, username, password)?;
        eprintln_tagged!("Finished": "fetching code; successfully saved");

        Ok(())
    }
}
