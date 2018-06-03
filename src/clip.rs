use std::fmt;

use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

define_error!();

#[derive(Debug)]
pub enum ErrorKind {
    GettingSourceFileFailed,
    CopyingClipboardFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.kind {
            ErrorKind::GettingSourceFileFailed => "failed to get source file.",
            ErrorKind::CopyingClipboardFailed => "failed to copy the source file to the clipboard.",
        };
        write!(b, "{}", message)
    }
}

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } =
        srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed)?;
    clip::copy_to_clipboard(file_name.as_ref()).chain(ErrorKind::CopyingClipboardFailed)
}
