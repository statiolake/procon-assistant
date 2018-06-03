use imp::clip;
use imp::srcfile;
use imp::srcfile::SrcFile;

define_error!();
define_error_kind! {
    [GettingSourceFileFailed; (); "failed to get source file."];
    [ReadingSourceFileFailed; (); "failed to read source file."];
}

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } =
        srcfile::get_source_file().chain(ErrorKind::GettingSourceFileFailed())?;
    println!(
        "{}",
        clip::read_source_file(file_name.as_ref(), true)
            .chain(ErrorKind::ReadingSourceFileFailed())?
    );

    Ok(())
}
