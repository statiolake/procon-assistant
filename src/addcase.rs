use crate::imp::common::open;
use crate::imp::config::ConfigFile;
use crate::imp::test_case::TestCaseFile;

define_error!();
define_error_kind! {
    [GettingConfigFailed; (); "failed to get user config."];
    [TestCaseFileSettingUpFailed; (); "failed to set up test case."];
    [TestCaseFileCreationFailed; (); "failed to create test case."];
    [FileOpeningFailed; (); "failed to open created file."];
}

pub fn main(_quiet: bool) -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config().chain(ErrorKind::GettingConfigFailed())?;
    let tsf = TestCaseFile::new_with_idx(
        TestCaseFile::next_unused_idx().chain(ErrorKind::TestCaseFileSettingUpFailed())?,
        "".as_bytes().into(),
        "".as_bytes().into(),
    );
    tsf.write().chain(ErrorKind::TestCaseFileCreationFailed())?;

    print_created!("{}, {}", tsf.if_name, tsf.of_name);

    open(&config, true, &[&tsf.if_name, &tsf.of_name]).chain(ErrorKind::FileOpeningFailed())?;

    Ok(())
}
