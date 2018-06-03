mod print_msg;

pub mod aoj;
pub mod atcoder;

use std::env;

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

pub fn main(args: Vec<String>) -> Result<()> {
    let arg = if args.is_empty() {
        handle_empty_arg()?
    } else {
        args.into_iter().next().unwrap()
    };

    let (contest_site, problem_id) = {
        let sp: Vec<_> = arg.splitn(2, ':').collect();

        if sp.len() != 2 {
            return Err(Error::new(ErrorKind::ArgumentFormatError(arg.clone())));
        }

        (sp[0], sp[1])
    };

    match contest_site {
        "aoj" => aoj::main(problem_id).chain(ErrorKind::ChildError()),
        "atcoder" | "at" => atcoder::main(problem_id).chain(ErrorKind::ChildError()),
        _ => Err(Error::new(ErrorKind::UnknownContestSite(
            contest_site.into(),
        ))),
    }
}

fn handle_empty_arg() -> Result<String> {
    let current_dir = env::current_dir().expect("critical error: failed to get current directory.");
    let maybe_dir_name = current_dir
        .file_name()
        .and_then(|x| x.to_str())
        .map(|x| x.to_string());

    if let Some(dir) = maybe_dir_name {
        let path = format!("{}", current_dir.display());
        if path.find("aoj").is_some() {
            return Ok(format!("aoj:{}", dir));
        } else if path.find("atcoder").is_some() {
            return Ok(format!("atcoder:{}", dir));
        }
    }

    Err(Error::new(ErrorKind::ProblemUnspecified()))
}
