use crate::imp::langs;
use crate::imp::preprocess;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get source file.")]
    GettingLanguageFailed { source: anyhow::Error },

    #[error("failed to read source file.")]
    ReadingSourceFileFailed { source: anyhow::Error },
}

pub fn main(quiet: bool) -> Result<()> {
    let lang = langs::get_lang().map_err(|e| Error::GettingLanguageFailed { source: e.into() })?;
    let src = preprocess::read_source_file(lang.src_file_name.as_ref())
        .and_then(|src| (lang.preprocessor)(quiet, src))
        .map_err(|e| Error::ReadingSourceFileFailed { source: e.into() })?;
    println!("{}", src);

    Ok(())
}
