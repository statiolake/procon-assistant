use crate::eprintln_tagged;
use crate::imp;
use crate::ExitStatus;
use anyhow::{bail, ensure};
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Initializes directory tree")]
pub struct InitDirs {
    #[clap(help = "The name of contest (the name of created directory)")]
    contest_name: String,

    #[clap(help = "The number of problems")]
    numof_problems: usize,

    #[clap(help = "The first problem character")]
    beginning_char: char,
}

impl InitDirs {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        // max number of problems is to avoid overflow of problem id.  we need
        // to limit the number of problems in order to avoid the problem id
        // exceeding `Z`.
        let max_numof_problems = match self.beginning_char {
            'a'..='z' => (self.beginning_char as u8 - b'a') as usize,
            'A'..='Z' => (self.beginning_char as u8 - b'A') as usize,
            ch => bail!("invalid start character: {}", ch),
        };

        ensure!(
            self.numof_problems <= max_numof_problems,
            "too many problems for contest starting with `{}`; max is {}, current is {}",
            self.beginning_char,
            max_numof_problems,
            self.numof_problems
        );

        // create directories
        imp::initdirs::create_directories(
            &self.contest_name,
            self.numof_problems,
            self.beginning_char,
        )
        .context("failed to create directories")?;
        eprintln_tagged!("Created": "directory tree");

        Ok(ExitStatus::Success)
    }
}
