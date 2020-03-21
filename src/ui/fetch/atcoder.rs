use crate::imp::auth::atcoder as auth;
use crate::imp::test_case::TestCaseFile;
use crate::ui::login::atcoder as login;
use scraper::{Html, Selector};
use std::result;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing tag: failed to find `{selector}`. maybe you are not logged in?")]
    FindingTagFailed { selector: String },

    #[error("unexpected number of <pre>: {detected}")]
    UnexpectedNumberOfPreTag { detected: usize },

    #[error("failed to determine test case file name.")]
    CouldNotDetermineTestCaseFileName { source: anyhow::Error },

    #[error("failed to get the page at `{url}`.")]
    AuthenticatedGetFailed { source: anyhow::Error, url: String },

    #[error("failed to get text from page.")]
    GettingTextFailed { source: anyhow::Error },

    #[error("invalid format for problem-id: `{problem}`. example: `abc022a` for AtCoder Beginner Contest 022 Problem A")]
    InvalidFormatForProblemId { problem: String },

    #[error("logging in failed.")]
    LoginFailed { source: anyhow::Error },
}

#[derive(Debug)]
pub struct AtCoder {
    problem: Problem,
}

impl AtCoder {
    pub fn new(problem_id: String) -> Result<AtCoder> {
        Problem::from_problem_id(problem_id).map(|problem| AtCoder { problem })
    }
}

#[derive(Debug)]
pub enum Problem {
    ProblemId {
        problem_id: String,
        contest_name: String,
        contest_id: String,
        problem: String,
        url: String,
    },
    DirectUrl {
        url: String,
    },
}

impl Problem {
    pub fn from_problem_id(problem_id: String) -> Result<Problem> {
        let problem = if problem_id.starts_with("http") {
            Problem::DirectUrl { url: problem_id }
        } else {
            if problem_id.len() != 7 {
                return Err(Error::InvalidFormatForProblemId {
                    problem: problem_id,
                });
            }
            let contest_name = problem_id[0..3].to_string();
            let contest_id = problem_id[0..6].to_string();
            let problem = problem_id[6..7].to_string();
            let url = format!(
                "https://beta.atcoder.jp/contests/{}/tasks/{}_{}",
                contest_id, contest_id, problem
            );
            Problem::ProblemId {
                problem_id,
                contest_name,
                contest_id,
                problem,
                url,
            }
        };
        Ok(problem)
    }

    pub fn problem_id(&self) -> &str {
        match *self {
            Problem::ProblemId { ref problem_id, .. } => &problem_id,
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

impl super::TestCaseProvider for AtCoder {
    fn site_name(&self) -> &str {
        "AtCoder"
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
            "needs_authenticate() is not implemetented for now, always returns `false'."
        );
        false
    }

    fn authenticate(&self, quiet: bool) -> result::Result<(), anyhow::Error> {
        login::main(quiet)
            .map_err(|e| Error::LoginFailed { source: e.into() })
            .map_err(Into::into)
    }

    fn fetch_test_case_files(
        &self,
        quiet: bool,
    ) -> result::Result<Vec<TestCaseFile>, anyhow::Error> {
        let text = download_text(quiet, self.problem.url())?;
        parse_text(text).map_err(Into::into)
    }
}

fn parse_text(text: String) -> Result<Vec<TestCaseFile>> {
    let document = Html::parse_document(&text);
    let sel_div_task_stmt = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let div_task_stmt = document.select(&sel_div_task_stmt).next();
    let div_task_stmt = div_task_stmt.ok_or_else(|| Error::FindingTagFailed {
        selector: "div#task-statement".into(),
    })?;

    let lang_ja = div_task_stmt.select(&sel_span_ja).next();
    let lang_ja = lang_ja.or_else(|| div_task_stmt.select(&sel_div_task_stmt).next());
    let lang_ja = lang_ja.ok_or_else(|| Error::FindingTagFailed {
        selector: "span.lang-ja".into(),
    })?;
    let pres: Vec<_> = lang_ja.select(&sel_pre).collect();

    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Error::UnexpectedNumberOfPreTag {
            detected: pres.len(),
        });
    }

    let beginning = TestCaseFile::next_unused_idx()
        .map_err(|e| Error::CouldNotDetermineTestCaseFileName { source: e.into() })?;
    let mut result = Vec::new();
    for i in 0..(pres.len() / 2) {
        let tsf = TestCaseFile::new_with_idx(
            beginning + i as i32,
            pres[i * 2 + 1].inner_html().as_bytes().into(),
            pres[i * 2 + 2].inner_html().as_bytes().into(),
        );
        result.push(tsf)
    }

    Ok(result)
}

fn download_text(quiet: bool, url: &str) -> Result<String> {
    auth::authenticated_get(quiet, url)
        .map_err(|e| Error::AuthenticatedGetFailed {
            source: e.into(),
            url: url.to_string(),
        })?
        .text()
        .map_err(|e| Error::GettingTextFailed { source: e.into() })
}
