use crate::eprintln_tagged;
use crate::imp::fetch::{ProblemDescriptor, TestCaseProvider};
use crate::imp::test_case::TestCaseFile;
use crate::ExitStatus;
use anyhow::bail;
use anyhow::{Context, Result};
use std::env;
use std::ffi::OsStr;

#[derive(clap::Parser)]
#[clap(about = "Fetches sample cases of a problem")]
pub struct Fetch {
    #[clap(help = "The problem-descriptor of the target problem. ex) aoj:0123, atcoder:abc012a")]
    problem_descriptor: Option<String>,
}

impl Fetch {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let dsc = parse_descriptor(self.problem_descriptor)
            .context("failed to parse problem descriptor")?;
        let provider = dsc.clone().resolve_provider().with_context(|| {
            format!(
                "failed to resolve problem provider from problem-descriptor `{:?}`",
                dsc
            )
        })?;
        let tcfs = fetch_test_case_files(provider)?;
        write_test_case_files(tcfs)?;

        Ok(ExitStatus::Success)
    }
}

pub fn fetch_test_case_files(provider: Box<dyn TestCaseProvider>) -> Result<Vec<TestCaseFile>> {
    eprintln_tagged!(
        "Fetching": "{} id {} (at {})",
        provider.site_name(),
        provider.problem_id(),
        provider.url()
    );

    let test_case_files = provider
        .fetch_test_case_files()
        .context("failed to fetch test case")?;

    Ok(test_case_files)
}

pub fn write_test_case_files(test_cases: Vec<TestCaseFile>) -> Result<()> {
    let num_test_cases = test_cases.len();
    for test_case in test_cases {
        eprintln_tagged!("Generating": "Sample Case: {}", test_case);
        test_case
            .write()
            .with_context(|| format!("failed to write test case file: `{}`", test_case))?;
    }
    eprintln_tagged!("Finished": "generating {} Sample Case(s)", num_test_cases);

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
        let problem_id_or_name = current_dir_name;
        let mut contest_id = None;

        for component in current_dir.components() {
            match component.as_os_str().to_str() {
                Some("aoj") => contest_site = Some("aoj".to_string()),
                Some("atcoder") | Some("at") => contest_site = Some("atcoder".to_string()),
                Some(other) if ["abc", "arc", "agc"].iter().any(|p| other.starts_with(p)) => {
                    contest_id = Some(other.to_string())
                }
                _ => continue,
            }
        }

        match (contest_site, contest_id) {
            (Some(contest_site), Some(contest_id)) => {
                let problem_name = problem_id_or_name;
                let problem_id = format!("{}{}", contest_id, problem_name);
                return Ok(ProblemDescriptor::new(contest_site, problem_id));
            }
            (Some(contest_site), None)
                if ["abc", "arc", "agc"]
                    .iter()
                    .any(|p| contest_site.starts_with(p)) =>
            {
                let problem_id = problem_id_or_name;
                return Ok(ProblemDescriptor::new(contest_site, problem_id));
            }
            _ => {}
        }
    }

    bail!("problem is not specified");
}

fn parse_descriptor(dsc: Option<String>) -> Result<ProblemDescriptor> {
    match dsc {
        Some(dsc) => ProblemDescriptor::parse(&dsc).context("failed parse problem descriptor"),
        None => handle_empty_arg(),
    }
}
