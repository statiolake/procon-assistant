use imp::common::open;
use imp::test_case::TestCaseFile;

use imp::config::ConfigFile;

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); "failed to get user config."];
    [TestCaseFileSettingUpFailed; (); "failed to set up test case."];
    [TestCaseFileCreationFailed; (); "failed to create test case."];
    [FileOpeningFailed; (); "failed to open created file."];
}

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;
    let tsf = TestCaseFile::new_with_next_unused_name("".as_bytes().into(), "".as_bytes().into())
        .chain(ErrorKind::TestCaseFileSettingUpFailed())?;
    tsf.write().chain(ErrorKind::TestCaseFileCreationFailed())?;

    print_created!("{}, {}", tsf.if_name, tsf.of_name);

    open(&config.editor, &tsf.if_name).chain(ErrorKind::FileOpeningFailed())?;
    open(&config.editor, &tsf.of_name).chain(ErrorKind::FileOpeningFailed())?;

    Ok(())
}
