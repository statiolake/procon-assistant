use crate::eprintln_tagged;
use crate::imp::auth;
use crate::imp::auth::atcoder as auth_atcoder;

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
        let (username, password) = auth::ask_account_info("AtCoder");
        eprintln_tagged!("Logging in": "to AtCoder");
        auth_atcoder::login(quiet, username, password)
            .map_err(|e| Error::LoginFailed { source: e.into() })?;
        eprintln_tagged!("Finished": "fetching code; successfully saved");

        Ok(())
    }
}
