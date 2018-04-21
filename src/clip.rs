use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

use Result;

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } = srcfile::get_source_file()?;
    clip::copy_to_clipboard(file_name.as_ref())
}
