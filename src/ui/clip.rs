use crate::imp::clip;
use crate::imp::langs;
use crate::imp::langs::Lang;
use crate::imp::preprocess;
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_tagged, eprintln_warning};
use std::path::Path;

#[derive(clap::Clap)]
#[clap(about = "Copies the source file to clipboard with your library expanded")]
pub struct Clip;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to copy to the clipboard")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to get source file")]
    GettingLanguageFailed { source: anyhow::Error },

    #[error("failed to copy the source file to the clipboard")]
    CopyingToClipboardFailed { source: anyhow::Error },
}

impl Clip {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang = langs::get_lang()
            .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
        copy_to_clipboard(quiet, &lang)
            .map_err(|e| Error(ErrorKind::CopyingToClipboardFailed { source: e.into() }))?;

        Ok(ExitStatus::Success)
    }
}

pub fn copy_to_clipboard(quiet: bool, lang: &Lang) -> preprocess::Result<()> {
    let file_path = Path::new(&lang.src_file_name);
    eprintln_tagged!("Copying": "{} to clipboard", file_path.display());
    let raw = preprocess::read_source_file(file_path)?;
    let preprocessed = (lang.preprocessor)(quiet, raw)?;
    let minified = (lang.minifier)(quiet, preprocessed);
    let lints = (lang.linter)(quiet, &minified);
    let minified = minified.into_inner() + "\n";
    clip::set_clipboard(minified.clone());
    eprintln_tagged!("Finished": "copying");

    if !quiet {
        eprintln_info!(
            "the copied string is as follows;  you can also pipe this to another program"
        );
        println!("{}", minified);
    }

    if !lints.is_empty() {
        eprintln_warning!("linter found {} errors, is this OK?", lints.len());

        for lint in lints {
            eprintln_tagged!("Lint": "{}", lint);
        }
    }

    Ok(())
}
