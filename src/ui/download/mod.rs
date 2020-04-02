pub mod atcoder;
pub mod local;

use self::atcoder::AtCoder;
use self::local::Local;
use crate::eprintln_tagged;
use crate::imp;
use crate::ui::fetch;
use crate::ui::fetch::TestCaseProvider;
use crate::ExitStatus;
use std::env;
use std::path::Path;
use std::result;
use std::str;

#[derive(clap::Clap)]
#[clap(about = "Fetches sample cases of all problems in a contest")]
pub struct Download {
    #[clap(help = "The contest-id of the target. ex) atcoder:abc012")]
    contest_id: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to download")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error(
        "argument's format is not correct: `{passed_arg}`;  example: `atcoder:abc022a` for AtCoder Beginner Contest 022 Problem A",
    )]
    ArgumentFormatError { passed_arg: String },

    #[error("contest-site `{site}` is unknown")]
    UnknownContestSite { site: String },

    #[error("failed to make fetchers")]
    MakingFetcherFailed { source: anyhow::Error },

    #[error("failed to fetch the problem")]
    FetchError { source: anyhow::Error },

    #[error("failed to create provider")]
    ProviderCreationFailed { source: anyhow::Error },

    #[error("failed to write test case file")]
    WritingTestCaseFailed { source: anyhow::Error },

    #[error("failed to create contest directories")]
    CreatingContestDirectoriesFailed { source: anyhow::Error },
}

impl Download {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let provider = match self.contest_id {
            Some(arg) => get_provider(arg),
            None => handle_empty_arg(),
        }?;

        let fetchers = provider
            .make_fetchers(quiet)
            .map_err(|source| Error(ErrorKind::MakingFetcherFailed { source }))?;

        eprintln_tagged!("Fetching": "{} (at {})", provider.contest_id(), provider.url());
        fetchers.prepare_generate()?;
        for (problem, fetcher) in fetchers.fetchers.into_iter().enumerate() {
            generate_one(
                quiet,
                fetchers.contest_id.clone(),
                fetchers.beginning_char,
                problem as u8,
                fetcher,
            )?;
        }

        Ok(ExitStatus::Success)
    }
}

fn provider_into_box<T: 'static + ContestProvider>(provider: T) -> Box<dyn ContestProvider> {
    Box::new(provider)
}

fn get_provider(arg: String) -> Result<Box<dyn ContestProvider>> {
    let (contest_site, contest_id) = parse_arg(&arg)?;
    match contest_site {
        "atcoder" | "at" => AtCoder::new(contest_id.to_string())
            .map_err(|e| Error(ErrorKind::ProviderCreationFailed { source: e.into() })),
        _ => Err(Error(ErrorKind::UnknownContestSite {
            site: contest_site.to_string(),
        })),
    }
    .map(provider_into_box)
}

fn get_local_provider() -> Result<Box<dyn ContestProvider>> {
    Ok(Box::new(Local::from_path("problems.txt".to_string())))
}

fn parse_arg(arg: &str) -> Result<(&str, &str)> {
    let sp: Vec<_> = arg.split(':').collect();
    if sp.len() != 2 {
        return Err(Error(ErrorKind::ArgumentFormatError {
            passed_arg: arg.to_string(),
        }));
    }
    Ok((sp[0], sp[1]))
}

fn handle_empty_arg() -> Result<Box<dyn ContestProvider>> {
    fn handle_empty_arg_impl() -> Option<Box<dyn ContestProvider>> {
        use std::ffi::OsStr;
        let current_dir = env::current_dir().ok()?;
        let file_name = current_dir
            .file_name()
            .and_then(OsStr::to_str)
            .map(ToString::to_string)?;

        if ["abc", "arc", "agc"].contains(&&file_name[0..3]) {
            AtCoder::new(file_name).map(provider_into_box).ok()
        } else {
            None
        }
    }
    Ok(handle_empty_arg_impl().unwrap_or(get_local_provider()?))
}

pub struct Fetchers {
    fetchers: Vec<Box<dyn TestCaseProvider>>,
    contest_id: String,
    beginning_char: char,
}

impl Fetchers {
    pub fn prepare_generate(&self) -> Result<()> {
        let numof_problems = self.fetchers.len();
        adjust_current_dir(&self.contest_id, self.beginning_char, numof_problems)?;

        Ok(())
    }
}

/// Generates directory tree if needed and ensure that we are in the contest directory.
fn adjust_current_dir(contest_id: &str, beginning_char: char, numof_problems: usize) -> Result<()> {
    let current_dir = env::current_dir().unwrap();
    let execuded_from_inside = matches!(current_dir.file_name(), Some(name) if name == contest_id);
    if execuded_from_inside {
        env::set_current_dir("..").unwrap();
    }

    imp::initdirs::create_directories(contest_id, numof_problems, beginning_char)
        .map_err(|e| Error(ErrorKind::CreatingContestDirectoriesFailed { source: e.into() }))?;

    env::set_current_dir(&Path::new(contest_id)).unwrap();

    Ok(())
}

pub fn generate_one(
    quiet: bool,
    mut contest_id: String,
    beginning_char: char,
    problem: u8,
    fetcher: Box<dyn TestCaseProvider + 'static>,
) -> Result<()> {
    let curr_actual = (beginning_char as u8 + problem) as char;
    env::set_current_dir(Path::new(&curr_actual.to_string())).unwrap();

    let curr_url = (b'a' + problem) as char;
    contest_id.push(curr_url);
    let tcfs = fetch::fetch_test_case_files(quiet, fetcher)
        .map_err(|e| Error(ErrorKind::FetchError { source: e.into() }))?;
    fetch::write_test_case_files(tcfs)
        .map_err(|e| Error(ErrorKind::WritingTestCaseFailed { source: e.into() }))?;
    contest_id.pop();

    env::set_current_dir(Path::new("..")).unwrap();

    Ok(())
}

pub trait ContestProvider {
    fn site_name(&self) -> &str;
    fn contest_id(&self) -> &str;
    fn url(&self) -> &str;

    fn make_fetchers(&self, quiet: bool) -> result::Result<Fetchers, anyhow::Error>;
}
