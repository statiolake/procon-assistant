use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

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
        fs::create_dir_all(&dir_path).context("failed to create directories")?;
        dir_path.pop();
    }

    Ok(())
}
