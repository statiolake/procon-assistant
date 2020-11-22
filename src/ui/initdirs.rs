use crate::eprintln_tagged;
use crate::imp::fs;
use crate::ExitStatus;
use anyhow::{bail, ensure};
use anyhow::{Context, Result};
use itertools::Itertools;

#[derive(clap::Clap)]
#[clap(about = "Initializes directory tree")]
pub struct InitDirs {
    #[clap(about = "The name of contest (the name of created directory)")]
    contest_name: String,

    #[clap(about = "The number of problems")]
    numof_problems: usize,

    #[clap(about = "The first problem character")]
    beginning_char: char,
}

impl InitDirs {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        validate_numof_problems(self.beginning_char, self.numof_problems)?;

        let dirnames = (0..self.numof_problems)
            .map(|idx| {
                (self.beginning_char as u8).checked_add(idx as u8).expect(
                    "critical error: problem char should not overflow as it was checked beforehand",
                )
            })
            .map(|p| (p as char).to_string())
            .collect_vec();

        // create directories
        fs::create_dirs(&self.contest_name, &dirnames, true)
            .context("failed to create directories")?;
        eprintln_tagged!("Created": "directory tree");

        Ok(ExitStatus::Success)
    }
}

/// Checks max number of problems to avoid overflow of problem. we need to limit
/// the number of problems in order to avoid the problem exceeding `Z`.
fn validate_numof_problems(beginning_char: char, numof_problems: usize) -> Result<()> {
    let max_numof_problems = match beginning_char {
        'a'..='z' => (b'z' - beginning_char as u8 + 1) as usize,
        'A'..='Z' => (b'Z' - beginning_char as u8 + 1) as usize,
        ch => bail!("invalid start character: {}", ch),
    };

    ensure!(
        numof_problems <= max_numof_problems,
        "too many problems for contest starting with `{}`; max is {}, current is {}",
        beginning_char,
        max_numof_problems,
        numof_problems
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_numof_problems() {
        assert!(validate_numof_problems('a', 26).is_ok());
        assert!(validate_numof_problems('z', 1).is_ok());
        assert!(validate_numof_problems('A', 26).is_ok());
        assert!(validate_numof_problems('Z', 1).is_ok());
        assert!(validate_numof_problems('a', 27).is_err());
        assert!(validate_numof_problems('z', 2).is_err());
        assert!(validate_numof_problems('A', 27).is_err());
        assert!(validate_numof_problems('Z', 2).is_err());
    }
}
