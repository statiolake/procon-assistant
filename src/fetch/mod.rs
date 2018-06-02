mod print_msg;

pub mod aoj;
pub mod atcoder;

use std::env;
use Error;
use Result;

pub fn main(args: Vec<String>) -> Result<()> {
    let arg = if args.is_empty() {
        handle_empty_arg()?
    } else {
        args.into_iter().next().unwrap()
    };

    let (contest_site, problem_id) = {
        let sp: Vec<_> = arg.splitn(2, ':').collect();

        if sp.len() != 2 {
            return Err(Error::new(
                "parsing contest-site and problem-id",
                "argument's format is not collect; please specify contest-site and problem-id separated by `:` (colon).",
            ));
        }

        (sp[0], sp[1])
    };

    match contest_site {
        "aoj" => aoj::main(problem_id),
        "atcoder" | "at" => atcoder::main(problem_id),
        _ => Err(Error::new(
            "processing contest-size",
            format!("the contest-site {} is not available.", contest_site),
        )),
    }
}

fn handle_empty_arg() -> Result<String> {
    if let Ok(current_dir) = env::current_dir() {
        if let Some(dir) = current_dir
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string())
        {
            let path = format!("{}", current_dir.display());
            if path.find("aoj").is_some() {
                return Ok(format!("aoj:{}", dir));
            } else if path.find("atcoder").is_some() {
                return Ok(format!("atcoder:{}", dir));
            }
        }
    }

    Err(Error::new(
        "parsing argument",
        "contest-site and problem-id are not specified.",
    ))
}
