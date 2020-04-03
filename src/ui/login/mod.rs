pub mod aoj;
pub mod atcoder;

use crate::ExitStatus;

#[derive(clap::Clap)]
#[clap(about = "Logs in to a contest-site")]
pub struct Login {
    #[clap(subcommand)]
    site: Site,
}

#[derive(clap::Clap)]
pub enum Site {
    #[clap(name = "atcoder", aliases = &["at"])]
    AtCoder(atcoder::AtCoder),

    #[clap(name = "aoj")]
    Aoj(aoj::Aoj),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("log in failed")]
    LoginError { source: anyhow::Error },
}

impl Site {
    fn run(self, quiet: bool) -> Result<()> {
        match self {
            Site::AtCoder(cmd) => cmd
                .run(quiet)
                .map_err(|e| Error::LoginError { source: e.into() }),
            Site::Aoj(cmd) => cmd
                .run(quiet)
                .map_err(|e| Error::LoginError { source: e.into() }),
        }
    }
}

impl Login {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        self.site.run(quiet)?;

        Ok(ExitStatus::Success)
    }
}

pub trait LoginUI {
    fn authenticate(&self, quiet: bool) -> anyhow::Result<()>;
}
