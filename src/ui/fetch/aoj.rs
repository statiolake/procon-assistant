use crate::imp::auth::aoj as auth;
use crate::imp::test_case::TestCaseFile;
use scraper::{Html, Selector};
use std::result;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to fetch the problem `{problem_id}`")]
    FetchingProblemFailed {
        source: anyhow::Error,
        problem_id: String,
    },

    #[error("unexpected number of <pre>: {detected}")]
    UnexpectedNumberOfPreTag { detected: usize },

    #[error("failed to determine test case file name")]
    CouldNotDetermineTestCaseFileName { source: anyhow::Error },

    #[error("failed to get the page at `{url}`")]
    AuthenticatedGetFailed { source: anyhow::Error, url: String },

    #[error("failed to get text from page")]
    GettingTextFailed { source: anyhow::Error },
}

#[derive(Debug)]
pub struct Aoj {
    problem: Problem,
}

impl Aoj {
    pub fn new(problem_id: String) -> Result<Aoj> {
        Problem::from_problem_id(problem_id).map(|problem| Aoj { problem })
    }
}

#[derive(Debug)]
pub enum Problem {
    ProblemId { problem_id: String, url: String },
    DirectUrl { url: String },
}

impl Problem {
    pub fn from_problem_id(problem_id: String) -> Result<Problem> {
        let problem = if problem_id.starts_with("http") {
            Problem::DirectUrl { url: problem_id }
        } else {
            Problem::ProblemId {
                url: format!(
                    "http://judge.u-aizu.ac.jp/onlinejudge/description.jsp?lang=jp&id={}",
                    problem_id,
                ),
                problem_id,
            }
        };
        Ok(problem)
    }

    pub fn problem_id(&self) -> &str {
        match *self {
            Problem::ProblemId { ref problem_id, .. } => problem_id,
            Problem::DirectUrl { .. } => "Unknown",
        }
    }

    pub fn url(&self) -> &str {
        match *self {
            Problem::ProblemId { ref url, .. } => url,
            Problem::DirectUrl { ref url } => url,
        }
    }
}

impl super::TestCaseProvider for Aoj {
    fn site_name(&self) -> &str {
        "Aizu Online Judge"
    }

    fn problem_id(&self) -> &str {
        self.problem.problem_id()
    }

    fn url(&self) -> &str {
        self.problem.url()
    }

    fn needs_authenticate(&self, quiet: bool) -> bool {
        print_info!(
            !quiet,
            "needs_authenticate() is not implemetented for now, always returns `false`"
        );
        false
    }

    fn authenticate(&self, quiet: bool) -> result::Result<(), anyhow::Error> {
        print_info!(
            !quiet,
            "authenticate() for AOJ is not implemented for now, do nothing"
        );
        Ok(())
    }

    fn fetch_test_case_files(
        &self,
        _quiet: bool,
    ) -> result::Result<Vec<TestCaseFile>, anyhow::Error> {
        let text = download_text(self.problem.url()).map_err(|e| Error::FetchingProblemFailed {
            source: e.into(),
            problem_id: self.problem.problem_id().to_string(),
        })?;

        parse_text(text).map_err(Into::into)
    }
}

pub fn parse_text(text: String) -> Result<Vec<TestCaseFile>> {
    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let mut pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 {
        return Err(Error::UnexpectedNumberOfPreTag {
            detected: pres.len(),
        });
    }

    if pres.len() % 2 == 1 {
        pres = pres.into_iter().skip(1).collect();
    }

    let mut result = Vec::new();
    let beginning = TestCaseFile::next_unused_idx()
        .map_err(|e| Error::CouldNotDetermineTestCaseFileName { source: e.into() })?;
    for i in 0..(pres.len() / 2) {
        let tsf = TestCaseFile::new_with_idx(
            beginning + i as i32,
            pres[i * 2].inner_html().as_bytes().into(),
            pres[i * 2 + 1].inner_html().as_bytes().into(),
        );
        result.push(tsf);
    }

    Ok(result)
}

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .map_err(|e| Error::AuthenticatedGetFailed {
            source: e.into(),
            url: url.to_string(),
        })?
        .text()
        .map_err(|e| Error::GettingTextFailed { source: e.into() })
}
