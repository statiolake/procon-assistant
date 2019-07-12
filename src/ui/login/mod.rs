pub mod atcoder;

define_error!();
define_error_kind! {
    [InvalidNumberOfArgument; (n: usize); format!(
        "the number of argumets are invalid: got {} but expected 1", n
    )];
    [UnknownContestSite; (site: String); format!(
        "contest-site {} is unknown.", site
    )];
    [LoginError; (); "log in failed.".to_string()];
}

pub fn main(quiet: bool, args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err(Error::new(ErrorKind::InvalidNumberOfArgument(args.len())));
    }

    match args[0].as_str() {
        "atcoder" | "at" => atcoder::main(quiet).chain(ErrorKind::LoginError()),
        _ => Err(Error::new(ErrorKind::UnknownContestSite(args[0].clone()))),
    }
}
