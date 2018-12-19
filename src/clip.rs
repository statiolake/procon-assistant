use std::path::Path;

use crate::imp::clip;
use crate::imp::langs;
use crate::imp::langs::Lang;
use crate::imp::preprocess;

define_error!();
define_error_kind! {
    [GettingLanguageFailed; (); "failed to get source file."];
    [CopyingToClipboardFailed; (); "failed to copy the source file to the clipboard."];
}

pub fn main(quiet: bool) -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    copy_to_clipboard(quiet, &lang).chain(ErrorKind::CopyingToClipboardFailed())
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
        "the copied string is as follows. you can pipe it when auto-copying did not function."
    );
    println!("{}", minified);

    if !lints.is_empty() {
        print_warning!("linter found {} errors. is this OK?", lints.len());

        for lint in lints {
            print_lint!("{}", lint);
        }
    }

    Ok(())
}
