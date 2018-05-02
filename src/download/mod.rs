mod print_msg;

pub mod atcoder;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

use {Error, Result};

use fetch;
use initdirs;

pub fn main(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return handle_empty_arg();
    }
    let arg = args.into_iter().next().unwrap();

    let (contest_site, contest_id) = {
        let sp: Vec<_> = arg.split(':').collect();

        if sp.len() != 2 {
            return Err(Error::new(
                "parsing contest-site and problem-id",
                "argument's format is not collect; please specify contest-site and problem-id separated by `:` (colon).",
            ));
        }

        (sp[0], sp[1])
    };

    match contest_site {
        "atcoder" | "at" => atcoder::main(contest_id),
        _ => Err(Error::new(
            "processing contest-size",
            format!("the contest-site {} is not available.", contest_site),
        )),
    }
}

fn handle_empty_arg() -> Result<()> {
    let problems = load_problem_list()?;
    initdirs::create_directories(".", 'a', problems.len() as u8);
    for (i, p) in problems.into_iter().enumerate() {
        let ch = 'a' as u8 + i as u8;
        env::set_current_dir(Path::new(unsafe { str::from_utf8_unchecked(&[ch]) })).unwrap();
        fetch::main(vec![p])?;
        env::set_current_dir("..").unwrap();
    }
    Ok(())
}

fn load_problem_list() -> Result<Vec<String>> {
    let problems_path = Path::new("problems.txt");
    if !problems_path.exists() {
        return Err(Error::new(
            "parsing argument",
            "contest-site and contest-id are not specified, and problems.txt is not found.",
        ));
    }
    let mut f = File::open(problems_path)
        .map_err(|e| Error::with_cause("reading problems.txt", "could not open it", box e))?;
    let mut content = String::new();
    f.read_to_string(&mut content)
        .map_err(|e| Error::with_cause("reading problems.txt", "could not read it", box e))?;
    let res: Vec<_> = content
        .split('\n')
        .filter(|&x| !x.starts_with("#"))
        .filter(|&x| x != "")
        .map(|x| x.into())
        .collect();

    Ok(res)
}
