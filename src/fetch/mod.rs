pub mod aoj;
pub mod atcoder;

use fetch::aoj::Aoj;
use fetch::atcoder::AtCoder;

use std::env;
use std::error;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::result;

use imp::test_case::TestCaseFile;
use tags::SPACER;

define_error!();
define_error_kind! {
    [ArgumentFormatError; (passed_arg: String); format!(concat!(
        "argument's format is not collect: `{}'.\n",
        "{}please specify contest-site and problem-id separated by `:' (colon)."
    ), passed_arg, SPACER)];
    [UnknownContestSite; (site: String); format!(
        "contest-site `{}' is unknown.", site
    )];
    [ProblemUnspecified; (); format!(
        "contest-site and problem-id are not specified."
    )];
    [FetchFailed; (); format!("failed to fetch.")];
    [ProviderCreationFailed; (); format!("failed to create provider.")];
    [TestCaseFileWritionFailed; (name: String); format!("failed to write test case file `{}'.", name)];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let dsc = get_descriptor(args.into_iter().next())?;
    let provider = get_provider(dsc)?;
    let tcfs = fetch_test_case_files(provider)?;
    write_test_case_files(tcfs)?;
    Ok(())
}

pub fn fetch_test_case_files(provider: Box<dyn TestCaseProvider>) -> Result<Vec<TestCaseFile>> {
    print_fetching!(
        "{} id {} (at {})",
        provider.site_name(),
        provider.problem_id(),
        provider.url()
    );

    if provider.needs_authenticate() {
        print_info!(true, "authentication is needed.");
        provider
            .authenticate()
            .map_err(|e| Error::with_cause(ErrorKind::ProviderCreationFailed(), e))?;
    }

    let test_case_files = provider
        .fetch_test_case_files()
        .map_err(|e| Error::with_cause(ErrorKind::FetchFailed(), e))?;
    Ok(test_case_files)
}

pub fn write_test_case_files(tcfs: Vec<TestCaseFile>) -> Result<()> {
    let n = tcfs
        .into_iter()
        .map(|tcf| {
            print_generating!("Sample Case: {}", tcf);
            tcf.write()
                .chain(ErrorKind::TestCaseFileWritionFailed(tcf.to_string()))
        })
        .count();
    print_finished!("generating {} Sample Case(s).", n);
    Ok(())
}

pub fn get_provider(dsc: ProblemDescriptor) -> Result<Box<dyn TestCaseProvider>> {
    match &*dsc.contest_site {
        "aoj" => Aoj::new(dsc.problem_id)
            .chain(ErrorKind::ProviderCreationFailed())
            .map(|p| (box p) as Box<_>),
        "atcoder" | "at" => AtCoder::new(dsc.problem_id)
            .chain(ErrorKind::ProviderCreationFailed())
            .map(|p| (box p) as Box<_>),
        _ => Err(Error::new(ErrorKind::UnknownContestSite(dsc.contest_site))),
    }
}
fn get_descriptor(problem_id: Option<String>) -> Result<ProblemDescriptor> {
    match problem_id {
        Some(arg) => ProblemDescriptor::parse(arg),
        None => handle_empty_arg(),
    }
}

fn handle_empty_arg() -> Result<ProblemDescriptor> {
    let current_dir = env::current_dir().expect("critical error: failed to get current directory.");

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

    Err(Error::new(ErrorKind::ProblemUnspecified()))
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
                return Err(Error::new(ErrorKind::ArgumentFormatError(dsc.clone())));
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

    fn needs_authenticate(&self) -> bool;
    fn authenticate(&self) -> result::Result<(), Box<dyn error::Error + Send>>;

    fn fetch_test_case_files(
        &self,
    ) -> result::Result<Vec<TestCaseFile>, Box<dyn error::Error + Send>>;
}
