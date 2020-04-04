pub mod atcoder;

use crate::imp::test_case::TestCase;
use anyhow::ensure;
use anyhow::Result;
use std::fmt::Debug;

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
            ensure!(sp.len() == 2, "invalid format for argument: {}", dsc);
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
    fn fetch_test_case_files(&self) -> Result<Vec<TestCase>>;
}
