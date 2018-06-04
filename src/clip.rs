use std::path::Path;

use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

define_error!();
define_error_kind! {
    [GettingSourceFileFailed; (); "failed to get source file."];
    [CopyingToClipboardFailed; (); "failed to copy the source file to the clipboard."];
}

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } =
        srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed())?;
    copy_to_clipboard(file_name.as_ref()).chain(ErrorKind::CopyingToClipboardFailed())
}

pub fn copy_to_clipboard(file_path: &Path) -> clip::Result<()> {
    print_copying!("{} to clipboard", file_path.display());
    let main_src = clip::read_source_file(file_path, false)?;
    let main_src = clip::preprocess(main_src)?;
    clip::set_clipboard(main_src);
    print_finished!("copying");
    Ok(())
}
