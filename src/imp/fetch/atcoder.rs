use super::TestCaseProvider;
use crate::eprintln_debug;
use crate::imp::auth::atcoder as auth;
use crate::imp::test_case::TestCase;
use easy_scraper::Pattern;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing tag: failed to find `{selector}`; are you successfully logged in?")]
    FindingTagFailed { selector: String },

    #[error("unexpected number of <pre>: {detected}")]
    UnexpectedNumberOfPreTag { detected: usize },

    #[error("failed to determine test case file name")]
    CouldNotDetermineTestCaseName { source: anyhow::Error },

    #[error("failed to get the page at `{url}`")]
    AuthenticatedGetFailed { source: anyhow::Error, url: String },

    #[error("failed to get text from page")]
    GettingTextFailed { source: anyhow::Error },

    #[error("invalid format for problem-id: `{problem}`;  example: `abc022a` for AtCoder Beginner Contest 022 Problem A")]
    InvalidFormatForProblemId { problem: String },

    #[error("logging in failed")]
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
        match self {
            Problem::ProblemId { problem_id, .. } => problem_id,
            Problem::DirectUrl { .. } => "Unknown",
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Problem::ProblemId { url, .. } => url,
            Problem::DirectUrl { url } => url,
        }
    }
}

impl TestCaseProvider for AtCoder {
    fn site_name(&self) -> &str {
        "AtCoder"
    }

    fn problem_id(&self) -> &str {
        self.problem.problem_id()
    }

    fn url(&self) -> &str {
        self.problem.url()
    }

    fn needs_authenticate(&self) -> bool {
        eprintln_debug!(
            "needs_authenticate() is not implemetented for now, always returns `false`"
        );

        false
    }

    fn fetch_test_case_files(&self) -> anyhow::Result<Vec<TestCase>> {
        let text = download_text(self.problem.url())?;
        parse_text(&text).map_err(Into::into)
    }
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

fn parse_text(text: &str) -> Result<Vec<TestCase>> {
    let pattern = Pattern::new(
        r#"
<div id="task-statement">
    <span class="lang-ja">
        <div class="part">
        <section>
            <h3>入力例 {{n}}</h3>
            <pre>
                {{input}}
            </pre>
        </section>
        </div>
        <div class="part">
        <section>
            <h3>出力例 {{n}}</h3>
            <pre>
                {{output}}
            </pre>
        </section>
        </div>
    </span>
</div>
"#,
    )
    .unwrap();

    let idx_start = TestCase::next_unused_idx()
        .map_err(|e| Error::CouldNotDetermineTestCaseName { source: e.into() })?;
    Ok(pattern
        .matches(text)
        .into_iter()
        .enumerate()
        .map(|(i, case)| {
            let idx = idx_start + i as i32;
            TestCase::new_with_idx(idx, case["input"].clone(), case["output"].clone())
        })
        .collect())
}
