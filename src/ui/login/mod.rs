pub mod aoj;
pub mod atcoder;

use crate::ExitStatus;
use anyhow::{Context, Result};

#[derive(clap::Parser)]
#[clap(about = "Logs in to a contest-site")]
pub struct Login {
    #[clap(subcommand)]
    site: Site,
}

#[derive(clap::Parser)]
pub enum Site {
    #[clap(name = "atcoder", aliases = &["at"])]
    AtCoder(atcoder::AtCoder),

    #[clap(name = "aoj")]
    Aoj(aoj::Aoj),
}

impl Site {
    fn run(self, quiet: bool) -> Result<()> {
        match self {
            Site::AtCoder(cmd) => cmd.run(quiet).context("failed to login"),
            Site::Aoj(cmd) => cmd.run(quiet).context("failed to login"),
        }
    }
}

impl Login {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        self.site.run(quiet)?;

        Ok(ExitStatus::Success)
    }
}

pub trait LoginUi {
    fn authenticate(&self, quiet: bool) -> Result<()>;
}
