use crate::imp::clip;
use crate::imp::langs;
use crate::imp::langs::Lang;
use crate::imp::preprocess;
use std::path::Path;

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

pub fn main(quiet: bool) -> Result<()> {
    let lang = langs::get_lang()
        .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
    copy_to_clipboard(quiet, &lang)
        .map_err(|e| Error(ErrorKind::CopyingToClipboardFailed { source: e.into() }))
}

pub fn copy_to_clipboard(quiet: bool, lang: &Lang) -> preprocess::Result<()> {
    let file_path = Path::new(&lang.src_file_name);
    print_copying!("{} to clipboard", file_path.display());
    let raw = preprocess::read_source_file(file_path)?;
    let preprocessed = (lang.preprocessor)(quiet, raw)?;
    let minified = (lang.minifier)(quiet, preprocessed);
    let lints = (lang.linter)(quiet, &minified);
    let minified = minified.into_inner() + "\n";
    clip::set_clipboard(minified.clone());
    print_finished!("copying");

    print_info!(
        !quiet,
        "the copied string is as follows;  you can also pipe this to another program"
    );
    println!("{}", minified);

    if !lints.is_empty() {
        print_warning!("linter found {} errors, is this OK?", lints.len());

        for lint in lints {
            print_lint!("{}", lint);
        }
    }

    Ok(())
}
