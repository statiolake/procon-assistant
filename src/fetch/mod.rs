mod print_msg;

pub mod aoj;
pub mod atcoder;

use std::env;
use std::ffi::OsStr;

define_error!();
define_error_kind! {
    [ArgumentFormatError; (passed_arg: String); format!(
        concat!(
            "argument's format is not collect: `{}'.\n",
            "please specify contest-site and problem-id separated by `:' (colon)."
        ),
        passed_arg
    )];
    [UnknownContestSite; (site: String); format!(
        "contest-site `{}' is unknown.", site
    )];
    [ProblemUnspecified; (); format!(
        "contest-site and problem-id are not specified."
    )];
    [ChildError; (); format!("during processing")];
}

// atcoder:abc092a
// ^^^^^^^ contest-site
//         ^^^^^^^ problem-id
//         ^^^ contest-name
//         ^^^^^^ contest-id
//               ^ problem

pub struct ContestSpecifier {
    contest_site: String,
    problem_id: String,
}

impl ContestSpecifier {
    pub fn new(contest_site: String, problem_id: String) -> ContestSpecifier {
        ContestSpecifier {
            contest_site,
            problem_id,
        }
    }

    pub fn parse(specifier: String) -> Result<ContestSpecifier> {
        let (contest_site, problem_id) = {
            let sp: Vec<_> = specifier.splitn(2, ':').collect();

            if sp.len() != 2 {
                return Err(Error::new(ErrorKind::ArgumentFormatError(
                    specifier.clone(),
                )));
            }

            (sp[0].to_string(), sp[1].to_string())
        };
        Ok(ContestSpecifier::new(contest_site, problem_id))
    }
}

pub fn main(args: Vec<String>) -> Result<()> {
    let specifier = match args.into_iter().next() {
        Some(arg) => ContestSpecifier::parse(arg),
        None => handle_empty_arg(),
    }?;

    match &*specifier.contest_site {
        "aoj" => aoj::main(&specifier.problem_id).chain(ErrorKind::ChildError()),
        "atcoder" | "at" => atcoder::main(&specifier.problem_id).chain(ErrorKind::ChildError()),
        _ => Err(Error::new(ErrorKind::UnknownContestSite(
            specifier.contest_site,
        ))),
    }
}

fn handle_empty_arg() -> Result<ContestSpecifier> {
    let current_dir = env::current_dir().expect("critical error: failed to get current directory.");

    // sometimes current directory has no name (for exampple: root directory)
    let maybe_current_dir_name = current_dir
        .file_name()
        .and_then(OsStr::to_str)
        .map(ToString::to_string);

    if let Some(current_dir_name) = maybe_current_dir_name {
        for component in current_dir.components() {
            return Ok(match component.as_os_str().to_str() {
                Some("aoj") => ContestSpecifier::new("aoj".to_string(), current_dir_name),
                Some("atcoder") | Some("at") => {
                    ContestSpecifier::new("atcoder".to_string(), current_dir_name)
                }
                _ => continue,
            });
        }
    }

    Err(Error::new(ErrorKind::ProblemUnspecified()))
}
