use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

use Result;

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } = srcfile::get_source_file()?;
    println!("{}", clip::read_source_file(file_name.as_ref(), true)?);

    Ok(())
}
