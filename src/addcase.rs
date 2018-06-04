use imp::common::open;
use imp::test_case::TestCaseFile;

use imp::config::ConfigFile;

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); "failed to get user config."];
    [TestCaseFileSettingUpFailed; (); "failed to set up testcase."];
    [TestCaseFileCreationFailed; (); "failed to create testcase."];
    [FileOpeningFailed; (); "failed to open created file."];
}

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;
    let tsf =
        TestCaseFile::new_with_next_unused_name().chain(ErrorKind::TestCaseFileSettingUpFailed())?;
    tsf.create().chain(ErrorKind::TestCaseFileCreationFailed())?;

    print_created!("{}, {}", tsf.infile_name, tsf.outfile_name);

    open(&config.editor, &tsf.infile_name).chain(ErrorKind::FileOpeningFailed())?;
    open(&config.editor, &tsf.outfile_name).chain(ErrorKind::FileOpeningFailed())?;

    Ok(())
}
