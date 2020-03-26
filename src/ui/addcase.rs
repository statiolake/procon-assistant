use crate::eprintln_tagged;
use crate::imp;
use crate::imp::config::ConfigFile;
use crate::imp::test_case::TestCase;

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
    TestCaseSettingUpFailed { source: anyhow::Error },

    #[error("failed to create test case")]
    TestCaseCreationFailed { source: anyhow::Error },

    #[error("failed to open created file")]
    FileOpeningFailed { source: anyhow::Error },
}

impl AddCase {
    pub fn run(self, _quiet: bool) -> Result<()> {
        let config: ConfigFile = ConfigFile::get_config()
            .map_err(|e| Error(ErrorKind::GettingConfigFailed { source: e.into() }))?;

        // Create empty test case
        let idx = TestCase::next_unused_idx()
            .map_err(|e| Error(ErrorKind::TestCaseSettingUpFailed { source: e.into() }))?;
        let tsf = TestCase::new_with_idx(idx, String::new(), String::new());

        // Write empty test case into file
        tsf.write()
            .map_err(|e| Error(ErrorKind::TestCaseCreationFailed { source: e.into() }))?;

        eprintln_tagged!("Created": "{}, {}", tsf.if_name, tsf.of_name);

        imp::common::open(&config, true, &[&tsf.if_name, &tsf.of_name])
            .map_err(|e| Error(ErrorKind::FileOpeningFailed { source: e.into() }))?;

        Ok(())
    }
}
