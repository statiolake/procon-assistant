pub mod atcoder;
pub mod local;

use self::atcoder::AtCoder;
use self::atcoder::Contest as AtCoderContest;
use self::local::Local;
use crate::eprintln_debug;
use crate::eprintln_tagged;
use crate::imp::fetch::TestCaseProvider;
use crate::ui::fetch;
use crate::ExitStatus;
use anyhow::bail;
use anyhow::{Context, Result};
use scopeguard::defer;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

#[derive(clap::Clap)]
#[clap(about = "Fetches sample cases of all problems in a contest")]
pub struct Download {
    #[clap(help = "The contest-id of the target. ex) atcoder:abc012")]
    contest_id: Option<String>,
}

impl Download {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let provider = match self.contest_id {
            Some(arg) => get_provider(arg),
            None => handle_empty_arg(),
        }?;

        let fetchers = provider
            .make_fetchers()
            .context("failed to make the fetcher")?;

        eprintln_tagged!("Fetching": "{} (at {})", provider.contest_id(), provider.url());
        fetchers.prepare_generate()?;
        eprintln_debug!("fetchers: {:?}", fetchers.fetchers);
        for fetcher in fetchers.fetchers {
            generate_one(&fetcher.problem, fetcher.fetcher)?;
        }

        Ok(ExitStatus::Success)
    }
}

fn get_provider(arg: String) -> Result<Box<dyn ContestProvider>> {
    let (contest_site, contest_id) = parse_arg(&arg)?;

    match contest_site {
        "atcoder" | "at" => {
            if contest_id.starts_with("http") {
                let contest = AtCoderContest::from_url(contest_id.to_string());
                let provider = AtCoder::new(contest);
                Ok(Box::new(provider) as _)
            } else {
                let contest = AtCoderContest::from_contest_id(contest_id.to_string())
                    .context("failed to parse contest-id")?;
                let provider = AtCoder::new(contest);
                Ok(Box::new(provider) as _)
            }
        }
        site => bail!("unknown contest site: `{}`", site),
    }
}

fn get_local_provider() -> Result<Box<dyn ContestProvider>> {
    Ok(Box::new(Local::from_path("problems.txt".to_string())))
}

fn parse_arg(arg: &str) -> Result<(&str, &str)> {
    let sp: Vec<_> = arg.splitn(2, ':').collect();

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
            AtCoderContest::from_contest_id(file_name)
                .map(|contest| Box::new(AtCoder::new(contest)) as _)
                .ok()
        } else {
            None
        }
    }

    Ok(handle_empty_arg_impl().unwrap_or(get_local_provider()?))
}

pub struct Fetcher {
    pub fetcher: Box<dyn TestCaseProvider>,
    pub problem: String,
}

impl fmt::Debug for Fetcher {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        b.debug_struct("Fetcher")
            .field("problem", &self.problem)
            .field("fetcher", &self.fetcher.url())
            .finish()
    }
}

pub struct Fetchers {
    fetchers: Vec<Fetcher>,
    contest_id: String,
}

impl Fetchers {
    pub fn prepare_generate(&self) -> Result<()> {
        let root = self.create_dirs().context("failed to create directories")?;
        env::set_current_dir(root)?;

        Ok(())
    }

    /// Create directories and return the root directory
    fn create_dirs(&self) -> Result<PathBuf> {
        let current_dir = env::current_dir().expect("critical error: failed to get current dir");
        let execuded_from_inside =
            matches!(current_dir.file_name(), Some(name) if name == &*self.contest_id);
        let root = if execuded_from_inside {
            Path::new("..")
        } else {
            Path::new(".")
        };

        let root = root.join(&self.contest_id);
        fs::create_dir_all(&root)?;
        for fetcher in &self.fetchers {
            fs::create_dir_all(root.join(&fetcher.problem))?;
        }

        Ok(root)
    }
}

pub fn generate_one(problem: &str, provider: Box<dyn TestCaseProvider>) -> Result<()> {
    env::set_current_dir(Path::new(problem)).expect("critical error: failed to chdir");
    defer! {
         env::set_current_dir("..").expect("critical error: failed to chdir");
    }

    let tcfs = fetch::fetch_test_case_files(provider).context("failed to read test cases")?;
    fetch::write_test_case_files(tcfs).context("failed to write test cases")?;

    Ok(())
}

pub trait ContestProvider {
    fn site_name(&self) -> &str;
    fn contest_id(&self) -> &str;
    fn url(&self) -> &str;
    fn make_fetchers(&self) -> Result<Fetchers>;
}
