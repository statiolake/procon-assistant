use std::fs;
use std::io;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    IOError { source: io::Error },
}

pub fn create_directories(
    contest_name: &str,
    numof_problems: usize,
    beginning_char: char,
) -> Result<()> {
    let mut dir_path = PathBuf::from(contest_name);
    for x in 0..numof_problems {
        let ch = (x as u8 + beginning_char as u8) as char;
        assert!(matches!(ch, 'a'..='z' | 'A'..='Z'));
        dir_path.push(ch.to_string());
        fs::create_dir_all(&dir_path).map_err(|source| Error::IOError { source })?;
        dir_path.pop();
    }

    Ok(())
}
