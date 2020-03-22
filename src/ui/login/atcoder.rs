use crate::imp::auth;
use crate::imp::auth::atcoder as auth_atcoder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to login")]
    LoginFailed { source: anyhow::Error },
}

pub fn main(quiet: bool) -> Result<()> {
    let (username, password) = auth::ask_account_info("AtCoder");
    print_logging_in!("to AtCoder");
    auth_atcoder::login(quiet, username, password)
        .map_err(|e| Error::LoginFailed { source: e.into() })?;
    print_finished!("fetching code; successfully saved");

    Ok(())
}
