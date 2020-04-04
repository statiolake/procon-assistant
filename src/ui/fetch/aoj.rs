use crate::eprintln_debug;
use crate::imp::auth::aoj as auth;
use crate::imp::test_case::TestCase;
use anyhow::ensure;
use anyhow::{Context, Result};
use scraper::{Html, Selector};

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

    fn needs_authenticate(&self) -> bool {
        eprintln_debug!(
            "needs_authenticate() is not implemetented for now, always returns `false`"
        );

        false
    }

    fn fetch_test_case_files(&self) -> Result<Vec<TestCase>> {
        let text = download_text(self.problem.url())
            .with_context(|| format!("failed to fetch a problem: {}", self.problem.problem_id()))?;

        parse_text(text).map_err(Into::into)
    }
}

pub fn parse_text(text: String) -> Result<Vec<TestCase>> {
    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let mut pres: Vec<_> = document.select(&sel_pre).collect();
    ensure!(
        pres.len() > 1,
        "unexpected number of <pre>: {} found",
        pres.len()
    );

    if pres.len() % 2 == 1 {
        pres = pres.into_iter().skip(1).collect();
    }

    let mut result = Vec::new();
    let beginning = TestCase::next_unused_idx().context("failed to get unused index")?;
    for i in 0..(pres.len() / 2) {
        let tsf = TestCase::new_with_idx(
            beginning + i as i32,
            pres[i * 2].inner_html(),
            pres[i * 2 + 1].inner_html(),
        );
        result.push(tsf);
    }

    Ok(result)
}

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .with_context(|| format!("failed to get {} with logged in", url))?
        .text()
        .context("failed to get text")
}
