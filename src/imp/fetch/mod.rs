pub mod atcoder;

use crate::imp::test_case::TestCase;
use std::fmt::Debug;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("argument's format is not collect: `{passed_arg}`;  example: `atcoder:abc022a` for AtCoder Beginner Contest 022 Problem A")]
    ArgumentFormatError { passed_arg: String },

    #[error("contest-site `{site}` is unknown")]
    UnknownContestSite { site: String },

    #[error("contest-site and problem-id are not specified")]
    ProblemUnspecified,

    #[error("failed to fetch")]
    FetchFailed { source: anyhow::Error },

    #[error("failed to create provider")]
    ProviderCreationFailed { source: anyhow::Error },

    #[error("failed to write test case file `{name}`")]
    TestCaseWritionFailed { source: anyhow::Error, name: String },
}

// atcoder:abc092a
// ^^^^^^^ contest-site
//         ^^^^^^^ problem-id
//         ^^^ contest-name
//         ^^^^^^ contest-id
//               ^ problem

pub struct ProblemDescriptor {
    pub contest_site: String,
    pub problem_id: String,
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
                return Err(Error::ArgumentFormatError { passed_arg: dsc });
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
    fn fetch_test_case_files(&self) -> anyhow::Result<Vec<TestCase>>;
}
