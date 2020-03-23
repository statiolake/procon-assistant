use crate::eprintln_tagged;
use crate::imp;

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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid beginning character `{ch}`; problem id must starts with [a-zA-Z]")]
    InvalidStartCharacter { ch: char },

    #[error("too many problems; the maximum number is {max} as the problem id starts with `{beginning}` but specified was {current}")]
    TooManyProblems {
        beginning: char,
        max: usize,
        current: usize,
    },

    #[error("error while creating directories")]
    CreatingDirectoryError { source: imp::initdirs::Error },
}

impl InitDirs {
    pub fn run(self, _quiet: bool) -> Result<()> {
        // max number of problems is to avoid overflow of problem id.  we need
        // to limit the number of problems in order to avoid the problem id
        // exceeding `Z`.
        let max_numof_problems = match self.beginning_char {
            'a'..='z' => (self.beginning_char as u8 - b'a') as usize,
            'A'..='Z' => (self.beginning_char as u8 - b'A') as usize,
            _ => {
                return Err(Error::InvalidStartCharacter {
                    ch: self.beginning_char,
                })
            }
        };

        if self.numof_problems > max_numof_problems {
            return Err(Error::TooManyProblems {
                beginning: self.beginning_char,
                max: max_numof_problems,
                current: self.numof_problems,
            });
        }

        // create directories
        imp::initdirs::create_directories(
            &self.contest_name,
            self.numof_problems,
            self.beginning_char,
        )
        .map_err(|source| Error::CreatingDirectoryError { source })?;
        eprintln_tagged!("Created": "directory tree");

        Ok(())
    }
}
