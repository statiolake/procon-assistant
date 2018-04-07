pub mod atcoder;

use Error;
use Result;

pub fn main(args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err(Error::new(
            "parsing argument",
            "the number of arguments is invalid: expected 1",
        ));
    }

    match args[0].as_str() {
        "atcoder" | "at" => atcoder::main(),
        _ => Err(Error::new(
            "parsing argument",
            format!("the specified contest-site {} is unknown.", args[0]),
        )),
    }
}
