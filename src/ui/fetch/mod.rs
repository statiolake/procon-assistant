use crate::imp::test_case::TestCase;
use crate::ui::fetch::aoj::Aoj;
use crate::ui::fetch::atcoder::AtCoder;
use crate::{eprintln_info, eprintln_tagged};
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::result;

pub mod aoj;
pub mod atcoder;

#[derive(clap::Clap)]
#[clap(about = "Fetches sample cases of a problem")]
pub struct Fetch {
    #[clap(help = "The problem-id of the target problem.  ex) aoj:0123, atcoder:abc012a")]
    problem_id: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("argument's format is not collect: `{passed_arg}`;  example: `atcoder:abc022a` for AtCoder Beginner Contest 022 Problem A")]
    ArgumentFormatError { passed_arg: String },

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
    pub fn run(self, quiet: bool) -> Result<()> {
        let dsc = get_descriptor(self.problem_id)?;
        let provider = get_provider(dsc)?;
        let tcfs = fetch_test_case_files(quiet, provider)?;
        write_test_case_files(tcfs)?;
        Ok(())
    }
}

pub fn fetch_test_case_files(
    quiet: bool,
    provider: Box<dyn TestCaseProvider>,
) -> Result<Vec<TestCase>> {
    eprintln_tagged!(
        "Fetching": "{} id {} (at {})",
        provider.site_name(),
        provider.problem_id(),
        provider.url()
    );

    if provider.needs_authenticate(quiet) {
        if !quiet {
            eprintln_info!("authentication is needed");
        }

        provider
            .authenticate(quiet)
            .map_err(|source| Error::ProviderCreationFailed { source })?;
    }

    let test_case_files = provider
        .fetch_test_case_files(quiet)
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

pub fn get_provider(dsc: ProblemDescriptor) -> Result<Box<dyn TestCaseProvider>> {
    match &*dsc.contest_site {
        "aoj" => Aoj::new(dsc.problem_id)
            .map_err(|e| Error::ProviderCreationFailed { source: e.into() })
            .map(provider_into_box),
        "atcoder" | "at" => AtCoder::new(dsc.problem_id)
            .map_err(|e| Error::ProviderCreationFailed { source: e.into() })
            .map(provider_into_box),
        _ => Err(Error::UnknownContestSite {
            site: dsc.contest_site,
        }),
    }
}

fn provider_into_box<T: 'static + TestCaseProvider>(x: T) -> Box<dyn TestCaseProvider> {
    Box::new(x)
}

fn get_descriptor(problem_id: Option<String>) -> Result<ProblemDescriptor> {
    match problem_id {
        Some(arg) => ProblemDescriptor::parse(arg),
        None => handle_empty_arg(),
    }
}

fn handle_empty_arg() -> Result<ProblemDescriptor> {
    let current_dir = env::current_dir().expect("critical error: failed to get current directory");

    // sometimes current directory has no name (for exampple: root directory)
    let maybe_current_dir_name = current_dir
        .file_name()
        .and_then(OsStr::to_str)
        .map(ToString::to_string);

    if let Some(current_dir_name) = maybe_current_dir_name {
        for component in current_dir.components() {
            return Ok(match component.as_os_str().to_str() {
                Some("aoj") => ProblemDescriptor::new("aoj".to_string(), current_dir_name),
                Some("atcoder") | Some("at") => {
                    ProblemDescriptor::new("atcoder".to_string(), current_dir_name)
                }
                _ => continue,
            });
        }
    }

    Err(Error::ProblemUnspecified)
}

// atcoder:abc092a
// ^^^^^^^ contest-site
//         ^^^^^^^ problem-id
//         ^^^ contest-name
//         ^^^^^^ contest-id
//               ^ problem

pub struct ProblemDescriptor {
    contest_site: String,
    problem_id: String,
}

impl ProblemDescriptor {
    pub fn new(contest_site: String, problem_id: String) -> ProblemDescriptor {
        ProblemDescriptor {
            contest_site,
            problem_id,
        }
    }

    pub fn parse(dsc: String) -> Result<ProblemDescriptor> {
        let (contest_site, problem_id) = {
            let sp: Vec<_> = dsc.splitn(2, ':').collect();

            if sp.len() != 2 {
                return Err(Error::ArgumentFormatError { passed_arg: dsc });
            }

            (sp[0].to_string(), sp[1].to_string())
        };
        Ok(ProblemDescriptor::new(contest_site, problem_id))
    }
}

pub trait TestCaseProvider: Debug {
    fn site_name(&self) -> &str;
    fn problem_id(&self) -> &str;
    fn url(&self) -> &str;
    fn needs_authenticate(&self, quiet: bool) -> bool;
    fn authenticate(&self, quiet: bool) -> result::Result<(), anyhow::Error>;
    fn fetch_test_case_files(&self, quiet: bool) -> result::Result<Vec<TestCase>, anyhow::Error>;
}
