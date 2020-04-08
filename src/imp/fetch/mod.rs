pub mod aoj;
pub mod atcoder;

use self::aoj::Aoj;
use self::atcoder::{AtCoder, Problem as AtCoderProblem};
use crate::imp::test_case::TestCase;
use anyhow::bail;
use anyhow::{Context, Result};
use std::fmt::Debug;

// atcoder:abc092a
// atcoder
// ^^^^^^^         contest-site
//               a
//               ^ problem-name
//         abc092
//         ^^^^^^  contest-id
//         abc
//         ^^^     contest-name
//         abc092a
//         ^^^^^^^ problem-id
// atcoder:abc092a
// ^^^^^^^^^^^^^^  contest-descriptor
// atcoder:abc092a
// ^^^^^^^^^^^^^^^ problem-descriptor

// aoj:0000
// aoj
// ^^^      contest-site
//     0000
//     ^^^^ problem-id
// aoj:0000
// ^^^^^^^^ problem-descriptor

#[derive(Debug, Clone)]
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

    pub fn parse(dsc: &str) -> Result<ProblemDescriptor> {
        let (contest_site, problem_id) = {
            let sp: Vec<_> = dsc.splitn(2, ':').collect();
            (sp[0].to_string(), sp[1].to_string())
        };

        Ok(ProblemDescriptor::new(contest_site, problem_id))
    }

    pub fn resolve_provider(self) -> Result<Box<dyn TestCaseProvider>> {
        match &*self.contest_site {
            "aoj" => Aoj::new(self.problem_id)
                .context("failed to create the provider Aoj")
                .map(|p| Box::new(p) as _),
            "atcoder" | "at" => AtCoderProblem::from_problem_id(self.problem_id)
                .context("failed to create the provider AtCoder")
                .map(|p| Box::new(AtCoder::new(p)) as _),
            other => bail!("unknown contest site: {}", other),
        }
    }
}

pub trait TestCaseProvider: Debug {
    fn site_name(&self) -> &str;
    fn problem_id(&self) -> &str;
    fn url(&self) -> &str;
    fn fetch_test_case_files(&self) -> Result<Vec<TestCase>>;
}
