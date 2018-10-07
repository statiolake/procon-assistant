use std::path::Path;

use imp::clip;
use imp::langs;
use imp::langs::Lang;
use imp::preprocess;

define_error!();
define_error_kind! {
    [GettingLanguageFailed; (); "failed to get source file."];
    [CopyingToClipboardFailed; (); "failed to copy the source file to the clipboard."];
}

pub fn main() -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    copy_to_clipboard(&lang).chain(ErrorKind::CopyingToClipboardFailed())
}

pub fn copy_to_clipboard(lang: &Lang) -> preprocess::Result<()> {
    let file_path = Path::new(&lang.src_file_name);
    print_copying!("{} to clipboard", file_path.display());
    let raw = preprocess::read_source_file(file_path)?;
    let preprocessed = (lang.preprocessor)(raw)?;
    let minified = (lang.minifier)(preprocessed);
    clip::set_clipboard(minified.into_inner());
    print_finished!("copying");
    Ok(())
}
