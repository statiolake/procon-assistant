use crate::imp::common::open;
use crate::imp::config::ConfigFile;
use crate::imp::test_case::TestCaseFile;

#[derive(clap::Clap)]
#[clap(about = "Adds a new test case;  creates `inX.txt` and `outX.txt` in the current directory")]
pub struct AddCase;

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to add a testcase")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to get user config")]
    GettingConfigFailed { source: anyhow::Error },

    #[error("failed to set up test case")]
    TestCaseFileSettingUpFailed { source: anyhow::Error },

    #[error("failed to create test case")]
    TestCaseFileCreationFailed { source: anyhow::Error },

    #[error("failed to open created file")]
    FileOpeningFailed { source: anyhow::Error },
}

impl AddCase {
    pub fn run(self, _quiet: bool) -> Result<()> {
        let config: ConfigFile = ConfigFile::get_config()
            .map_err(|e| Error(ErrorKind::GettingConfigFailed { source: e.into() }))?;
        let tsf = TestCaseFile::new_with_idx(
            TestCaseFile::next_unused_idx()
                .map_err(|e| Error(ErrorKind::TestCaseFileSettingUpFailed { source: e.into() }))?,
            Vec::new(),
            Vec::new(),
        );
        tsf.write()
            .map_err(|e| Error(ErrorKind::TestCaseFileCreationFailed { source: e.into() }))?;

        print_created!("{}, {}", tsf.if_name, tsf.of_name);

        open(&config, true, &[&tsf.if_name, &tsf.of_name])
            .map_err(|e| Error(ErrorKind::FileOpeningFailed { source: e.into() }))?;

        Ok(())
    }
}
