use std::fs;
use std::path::PathBuf;

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

        create_directories(
            &*self.contest_name,
            self.beginning_char,
            self.numof_problems,
        );

        Ok(())
    }
}

pub fn create_directories(contest_name: &str, beginning_char: char, numof_problems: usize) {
    let mut dir_path = PathBuf::from(contest_name);
    for ch in (0..numof_problems).map(|x| (x as u8 + beginning_char as u8) as char) {
        dir_path.push(ch.to_string());
        fs::create_dir_all(&dir_path).unwrap();
        dir_path.pop();
    }

    print_created!("directory tree");
}
