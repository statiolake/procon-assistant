use crate::imp::fetch::atcoder::AtCoder;
use crate::imp::fetch::{ProblemDescriptor, TestCaseProvider};
use crate::imp::test_case::TestCase;
use crate::ui::fetch::aoj::Aoj;
use crate::ui::login;
use crate::ui::login::LoginUI;
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_tagged};
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;

pub mod aoj;

#[derive(clap::Clap)]
#[clap(about = "Fetches sample cases of a problem")]
pub struct Fetch {
    #[clap(help = "The problem-id of the target problem.  ex) aoj:0123, atcoder:abc012a")]
    problem_id: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("contest-site `{site}` is unknown")]
    UnknownContestSite { site: String },

    #[error("contest-site and problem-id are not specified")]
    ProblemUnspecified,

    #[error("failed to fetch")]
    FetchFailed { source: anyhow::Error },

    #[error("failed to create provider")]
    ProviderCreationFailed { source: anyhow::Error },

    #[error("failed to write test case file `{name}`")]
    TestCaseWritionFailed { source: anyhow::Error, name: String },
}

impl Fetch {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let dsc = get_descriptor(self.problem_id)?;
        let (provider, login_ui) = get_provider(dsc)?;
        let tcfs = fetch_test_case_files(quiet, provider, login_ui)?;
        write_test_case_files(tcfs)?;

        Ok(ExitStatus::Success)
    }
}

pub fn fetch_test_case_files(
    quiet: bool,
    provider: Box<dyn TestCaseProvider>,
    login_ui: Box<dyn LoginUI>,
) -> Result<Vec<TestCase>> {
    eprintln_tagged!(
        "Fetching": "{} id {} (at {})",
        provider.site_name(),
        provider.problem_id(),
        provider.url()
    );

    if provider.needs_authenticate() {
        if !quiet {
            eprintln_info!("authentication is needed");
        }

        login_ui
            .authenticate(quiet)
            .map_err(|source| Error::ProviderCreationFailed { source })?;
    }

    let test_case_files = provider
        .fetch_test_case_files()
        .map_err(|source| Error::FetchFailed { source })?;

    Ok(test_case_files)
}

pub fn write_test_case_files(tcfs: Vec<TestCase>) -> Result<()> {
    let sample_cases = tcfs.len();
    for tcf in tcfs {
        eprintln_tagged!("Generating": "Sample Case: {}", tcf);
        tcf.write().map_err(|e| Error::TestCaseWritionFailed {
            source: e.into(),
            name: tcf.to_string(),
        })?;
    }
    eprintln_tagged!("Finished": "generating {} Sample Case(s)", sample_cases);

    Ok(())
}

fn handle_empty_arg() -> Result<ProblemDescriptor> {
    let current_dir = env::current_dir().expect("critical error: failed to get current directory");

    // sometimes current directory has no name (for exampple: root directory)
    let maybe_current_dir_name = current_dir
        .file_name()
        .and_then(OsStr::to_str)
        .map(ToString::to_string);

    if let Some(current_dir_name) = maybe_current_dir_name {
        let mut contest_site = None;
        let problem = current_dir_name;
        let mut contest_id = None;

        for component in current_dir.components() {
            match component.as_os_str().to_str() {
                Some("aoj") => contest_site = Some("aoj".to_string()),
                Some("atcoder") | Some("at") => contest_site = Some("atcoder".to_string()),
                Some(other)
                    if other.starts_with("abc")
                        || other.starts_with("arc")
                        || other.starts_with("agc") =>
                {
                    contest_id = Some(other.to_string())
                }
                _ => continue,
            }
        }

        match (contest_site, contest_id) {
            (Some(contest_site), Some(contest_id)) => {
                return Ok(ProblemDescriptor::new(
                    contest_site,
                    format!("{}{}", contest_id, problem),
                ))
            }
            (Some(contest_site), None)
                if problem.starts_with("abc")
                    || problem.starts_with("arc")
                    || problem.starts_with("agc") =>
            {
                return Ok(ProblemDescriptor::new(contest_site, problem))
            }
            _ => {}
        }
    }

    Err(Error::ProblemUnspecified)
}

pub fn get_provider(
    dsc: ProblemDescriptor,
) -> Result<(Box<dyn TestCaseProvider>, Box<dyn LoginUI>)> {
    match &*dsc.contest_site {
        "aoj" => Aoj::new(dsc.problem_id)
            .map_err(|e| Error::ProviderCreationFailed { source: e.into() })
            .map(|t| (t, login::aoj::Aoj))
            .map(provider_into_box),
        "atcoder" | "at" => AtCoder::new(dsc.problem_id)
            .map_err(|e| Error::ProviderCreationFailed { source: e.into() })
            .map(|t| (t, login::atcoder::AtCoder))
            .map(provider_into_box),
        _ => Err(Error::UnknownContestSite {
            site: dsc.contest_site,
        }),
    }
}

fn provider_into_box<T: 'static + TestCaseProvider, L: 'static + LoginUI>(
    (t, l): (T, L),
) -> (Box<dyn TestCaseProvider>, Box<dyn LoginUI>) {
    (Box::new(t), Box::new(l))
}

fn get_descriptor(problem_id: Option<String>) -> Result<ProblemDescriptor> {
    match problem_id {
        Some(arg) => ProblemDescriptor::parse(arg)
            .map_err(|e| Error::ProviderCreationFailed { source: e.into() }),
        None => handle_empty_arg(),
    }
}
