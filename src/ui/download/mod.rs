use crate::eprintln_debug;
use crate::eprintln_tagged;
use crate::imp::download::ContestDescriptor;
use crate::imp::fetch::TestCaseProvider;
use crate::ui::fetch;
use crate::ExitStatus;
use anyhow::{Context, Result};
use scopeguard::defer;
use std::env;
use std::path::Path;
use std::str;

#[derive(clap::Clap)]
#[clap(about = "Fetches sample cases of all problems in a contest")]
pub struct Download {
    #[clap(help = "The contest-descriptor of the target. ex) atcoder:abc012")]
    contest_descriptor: Option<String>,
}

impl Download {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let dsc = parse_descriptor(self.contest_descriptor)
            .context("failed to parse contest-descriptor")?;
        let contest_provider = dsc.clone().resolve_provider().with_context(|| {
            format!(
                "failed to resolve provider from contest-descriptor `{:?}`",
                dsc
            )
        })?;
        let fetchers = contest_provider
            .make_fetchers()
            .context("failed to make the fetcher")?;

        eprintln_tagged!("Fetching": "{} (at {})", contest_provider.contest_id(), contest_provider.url());
        fetchers.prepare_generate()?;
        eprintln_debug!("fetchers: {:?}", fetchers.fetchers);
        for fetcher in fetchers.fetchers {
            generate_one(&fetcher.problem_name, fetcher.provider)?;
        }

        Ok(ExitStatus::Success)
    }
}

pub fn generate_one(problem: &str, provider: Box<dyn TestCaseProvider>) -> Result<()> {
    // Chdir to the directory for individual problem. Use defer! to ensure that the current
    // directory is restored before ending.
    let current_dir = env::current_dir().expect("critical error: failed to get current directory");
    env::set_current_dir(Path::new(problem)).expect("critical error: failed to chdir");
    defer! {
         env::set_current_dir(current_dir).expect("critical error: failed to chdir");
    }

    let test_cases = fetch::fetch_test_case_files(provider).context("failed to read test cases")?;
    fetch::write_test_case_files(test_cases).context("failed to write test cases")?;

    Ok(())
}

fn handle_empty_arg() -> Result<ContestDescriptor> {
    fn handle_empty_arg_impl() -> Option<ContestDescriptor> {
        use std::ffi::OsStr;
        let current_dir = env::current_dir().ok()?;
        let file_name = current_dir
            .file_name()
            .and_then(OsStr::to_str)
            .map(ToString::to_string)?;

        if ["abc", "arc", "agc"].contains(&&file_name[0..3]) {
            Some(ContestDescriptor::new("atcoder".to_string(), file_name))
        } else {
            None
        }
    }

    let dsc = handle_empty_arg_impl()
        .unwrap_or_else(|| ContestDescriptor::new("local".to_string(), "problems.txt".to_string()));

    Ok(dsc)
}

fn parse_descriptor(dsc: Option<String>) -> Result<ContestDescriptor> {
    match dsc {
        Some(dsc) => ContestDescriptor::parse(&dsc).context("failed to parse contest-descriptor"),
        None => handle_empty_arg(),
    }
}
