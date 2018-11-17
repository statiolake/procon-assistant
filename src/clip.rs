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
    let minified = minified.into_inner() + "\n";
    clip::set_clipboard(minified.clone());
    print_finished!("copying");
    print_info!(
        true,
        "the copied string is as follows. you can pipe it when auto-copying did not function."
    );
    println!("{}", minified);
    Ok(())
}
