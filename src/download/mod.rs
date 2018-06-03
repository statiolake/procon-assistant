mod print_msg;

pub mod atcoder;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

use fetch;
use initdirs;

define_error!();
define_error_kind! {
    [ArgumentFormatError; (passed_arg: String); format!(
        concat!(
            "argument's format is not collect: `{}'.\n",
            "please specify contest-site and problem-id separated by `:' (colon)."
        ),
        passed_arg
    )];
    [UnknownContestSite; (site: String); format!("contest-site `{}' is unknown.", site)];
    [AnythingNotSpecified; (); format!(
        "contest-site and contest-id are not specified, and problems.txt is not found."
    )];
    [CouldNotOpenProblemsTxt; (); format!(
        "problems.txt found, but could not open problems.txt."
    )];
    [CouldNotReadProblemsTxt; (); "could not read problems.txt.".to_string()];
    [FetchError; (); "failed to fetch the problem".to_string()];
    [ChildError; (); "during processing".to_string()];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let arg = if args.is_empty() {
        match handle_empty_arg() {
            Some(arg) => arg,
            None => return load_local_contest(),
        }
    } else {
        args.into_iter().next().unwrap()
    };

    let (contest_site, contest_id) = {
        let sp: Vec<_> = arg.split(':').collect();

        if sp.len() != 2 {
            return Err(Error::new(ErrorKind::ArgumentFormatError(arg.clone())));
        }

        (sp[0], sp[1])
    };

    match contest_site {
        "atcoder" | "at" => atcoder::main(contest_id).chain(ErrorKind::ChildError()),
        _ => Err(Error::new(ErrorKind::UnknownContestSite(
            contest_site.to_string(),
        ))),
    }
}

fn handle_empty_arg() -> Option<String> {
    env::current_dir()
        .ok()
        .and_then(|current_dir| {
            current_dir
                .file_name()
                .and_then(|x| x.to_str())
                .map(|x| x.to_string())
        })
        .and_then(|file_name| {
            if file_name.starts_with("abc") || file_name.starts_with("arc")
                || file_name.starts_with("agc")
            {
                Some(format!("at:{}", file_name))
            } else {
                None
            }
        })
}

fn load_local_contest() -> Result<()> {
    let problems = load_problem_list()?;
    initdirs::create_directories(".", 'a', problems.len() as u8);
    for (i, p) in problems.into_iter().enumerate() {
        let ch = 'a' as u8 + i as u8;
        env::set_current_dir(Path::new(unsafe { str::from_utf8_unchecked(&[ch]) })).unwrap();
        fetch::main(vec![p]).chain(ErrorKind::FetchError())?;
        env::set_current_dir("..").unwrap();
    }
    Ok(())
}

fn load_problem_list() -> Result<Vec<String>> {
    let problems_path = Path::new("problems.txt");
    if !problems_path.exists() {
        return Err(Error::new(ErrorKind::AnythingNotSpecified()));
    }
    let mut f = File::open(problems_path).chain(ErrorKind::CouldNotOpenProblemsTxt())?;
    let mut content = String::new();
    f.read_to_string(&mut content)
        .chain(ErrorKind::CouldNotReadProblemsTxt())?;
    let res: Vec<_> = content
        .split('\n')
        .map(|x| x.trim())
        .filter(|&x| !x.starts_with("#"))
        .filter(|&x| x != "")
        .map(|x| x.into())
        .collect();

    Ok(res)
}
