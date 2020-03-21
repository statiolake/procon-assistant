pub mod atcoder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the number of argumets are invalid: got {n} but expected 1")]
    InvalidNumberOfArgument { n: usize },

    #[error("contest-site {site} is unknown")]
    UnknownContestSite { site: String },

    #[error("log in failed")]
    LoginError { source: anyhow::Error },
}

pub fn main(quiet: bool, args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err(Error::InvalidNumberOfArgument { n: args.len() });
    }

    match args[0].as_str() {
        "atcoder" | "at" => {
            atcoder::main(quiet).map_err(|e| Error::LoginError { source: e.into() })
        }
        _ => Err(Error::UnknownContestSite {
            site: args[0].clone(),
        }),
    }
}
