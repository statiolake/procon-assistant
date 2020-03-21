use std::fs;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid number of arguments for initdir command: {n}: {info}")]
    InvalidNumberOfArgument { n: usize, info: &'static str },

    #[error("failed to parse the number of problems")]
    ParsingNumberOfProblemsFailed { source: anyhow::Error },
}

pub fn main(_quiet: bool, args: Vec<String>) -> Result<()> {
    let beginning_char = match args.len() {
        0 => Err(Error::InvalidNumberOfArgument {
            n: 0,
            info: "please specify contest-name and the number of problems",
        }),
        1 => Err(Error::InvalidNumberOfArgument {
            n: 1,
            info: "please specify the number of problems",
        }),
        2 => Ok('a'),
        3 if !args[2].is_empty() => Ok(args[2].chars().next().unwrap()),
        n => Err(Error::InvalidNumberOfArgument {
            n,
            info: "too many arguments",
        }),
    }?;

    let contest_name = args[0].as_str();
    let numof_problems: u8 = args[1]
        .parse::<u8>()
        .map_err(|e| Error::ParsingNumberOfProblemsFailed { source: e.into() })?;

    create_directories(contest_name, beginning_char, numof_problems);

    Ok(())
}

pub fn create_directories(contest_name: &str, beginning_char: char, numof_problems: u8) {
    let mut dir_path = PathBuf::from(contest_name);
    for ch in (0..numof_problems).map(|x| (x + beginning_char as u8) as char) {
        dir_path.push(ch.to_string());
        fs::create_dir_all(&dir_path).unwrap();
        dir_path.pop();
    }
    print_created!("directory tree");
}
