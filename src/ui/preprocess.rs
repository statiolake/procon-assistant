use crate::imp::langs;
use crate::imp::langs::Preprocessed;
use crate::ExitStatus;

#[derive(clap::Clap)]
#[clap(about = "Preprocesses current source file and prepares to submit")]
pub struct Preprocess;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to guess a current language")]
    GuessingLanguageFailed { source: anyhow::Error },

    #[error("failed to read source file")]
    ReadingSourceFileFailed { source: anyhow::Error },

    #[error("failed to preprocess the source")]
    PreprocessFailed { source: anyhow::Error },
}

impl Preprocess {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_language()
            .map_err(|e| Error::GuessingLanguageFailed { source: e.into() })?;
        let source = lang
            .get_source()
            .map_err(|source| Error::ReadingSourceFileFailed { source })?;
        let Preprocessed(preprocessed) = lang
            .preprocess(&source)
            .map_err(|source| Error::PreprocessFailed { source })?;

        println!("{}", preprocessed);

        Ok(ExitStatus::Success)
    }
}
