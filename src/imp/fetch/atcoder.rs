use super::TestCaseProvider;
use crate::eprintln_debug;
use crate::imp::auth::atcoder as auth;
use crate::imp::test_case::TestCase;
use anyhow::ensure;
use anyhow::{Context, Result};
use easy_scraper::Pattern;

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
            ensure!(
                problem_id.len() == 7,
                "invalid format for problem id: {}",
                problem_id
            );

            let contest_name = problem_id[0..3].to_string();
            let contest_id = problem_id[0..6].to_string();
            let problem = problem_id[6..7].to_string();
            let url = format!(
                "https://atcoder.jp/contests/{}/tasks/{}_{}",
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

    fn fetch_test_case_files(&self) -> Result<Vec<TestCase>> {
        let text = download_text(self.problem.url())?;
        parse_text(&text).map_err(Into::into)
    }
}

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .with_context(|| format!("failed to get `{}` with login", url))?
        .text()
        .context("failed to get the text")
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

    let idx_start = TestCase::next_unused_idx().context("failed to get unused index")?;
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
