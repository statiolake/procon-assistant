use imp::langs;
use imp::preprocess;

define_error!();
define_error_kind! {
    [GettingLanguageFailed; (); "failed to get source file."];
    [ReadingSourceFileFailed; (); "failed to read source file."];
}

pub fn main() -> Result<()> {
    let lang = langs::get_lang().chain(ErrorKind::GettingLanguageFailed())?;
    let src = preprocess::read_source_file(lang.src_file_name.as_ref())
        .and_then(|src| (lang.preprocessor)(src))
        .chain(ErrorKind::ReadingSourceFileFailed())?;
    println!("{}", src);

    Ok(())
}
