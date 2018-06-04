use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

define_error!();
define_error_kind! {
    [GettingSourceFileFailed; (); "failed to get source file."];
    [CopyingClipboardFailed; (); "failed to copy the source file to the clipboard."];
}

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } =
        srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed())?;
    clip::copy_to_clipboard(file_name.as_ref()).chain(ErrorKind::CopyingClipboardFailed())
}
