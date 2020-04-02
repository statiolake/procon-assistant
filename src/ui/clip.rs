use crate::imp::clip;
use crate::imp::langs;
use crate::imp::langs::{Language, Minified, Preprocessed};
use crate::ui::CONFIG;
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_tagged, eprintln_warning};

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

    #[error("failed to load source file")]
    LoadingSourceFileFailed { source: anyhow::Error },

    #[error("failed to preprocess the source")]
    PreprocessFailed { source: anyhow::Error },

    #[error("failed to minify the preprocessed source")]
    MinifyFailed { source: anyhow::Error },

    #[error("failed to copy the source file to the clipboard")]
    CopyingToClipboardFailed { source: anyhow::Error },
}

impl Clip {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_language()
            .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
        copy_to_clipboard(quiet, &*lang)
            .map_err(|e| Error(ErrorKind::CopyingToClipboardFailed { source: e.into() }))?;

        Ok(ExitStatus::Success)
    }
}

pub fn copy_to_clipboard<L: Language + ?Sized>(quiet: bool, lang: &L) -> Result<()> {
    eprintln_tagged!("Copying": "source file to clipboard");
    let source = lang
        .get_source()
        .map_err(|source| Error(ErrorKind::LoadingSourceFileFailed { source }))?;
    let preprocessed = lang
        .preprocess(&source)
        .map_err(|source| Error(ErrorKind::PreprocessFailed { source }))?;
    let minified = if CONFIG.clip.minify {
        lang.minify(&preprocessed)
            .map_err(|source| Error(ErrorKind::MinifyFailed { source }))?
    } else {
        let Preprocessed(p) = preprocessed;
        Minified(p)
    };

    let lints = lang.lint(&minified);
    let Minified(minified) = minified;
    let minified = minified + "\n";
    clip::set_clipboard(minified.clone());
    eprintln_tagged!("Finished": "copying");

    if !quiet {
        eprintln_info!("the copied string is as follows:");
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
