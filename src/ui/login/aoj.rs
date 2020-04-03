use super::LoginUI;

#[derive(clap::Clap)]
#[clap(about = "Logs in to Aoj")]
pub struct Aoj;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to login")]
    LoginFailed { source: anyhow::Error },
}

impl Aoj {
    pub fn run(self, quiet: bool) -> Result<()> {
        Aoj.authenticate(quiet)
            .map_err(|e| Error::LoginFailed { source: e.into() })
    }
}

impl LoginUI for Aoj {
    fn authenticate(&self, _quiet: bool) -> anyhow::Result<()> {
        // TODO: implement
        Ok(())
    }
}
