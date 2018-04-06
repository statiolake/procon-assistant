mod print_msg;

pub mod atcoder;

use Error;
use Result;

pub fn main(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(Error::new(
            "parsing argument",
            "contest-site and contest-id are not specified.",
        ));
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
        "atcoder" => atcoder::main(contest_id),
        _ => Err(Error::new(
            "processing contest-size",
            format!("the contest-site {} is not available.", contest_site),
        )),
    }
}