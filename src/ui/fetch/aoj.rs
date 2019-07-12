use scraper::{Html, Selector};

use std::error;
use std::result;

use crate::imp::auth::aoj as auth;
use crate::imp::test_case::TestCaseFile;

define_error!();
define_error_kind! {
    [FetchingProblemFailed; (problem_id: String); format!("failed to fetch the problem `{}'", problem_id)];
    [UnexpectedNumberOfPreTag; (detected: usize); format!("unexpected number of <pre>: {}", detected)];
    [CouldNotDetermineTestCaseFileName; (); "failed to determine test case file name.".to_string()];
    [AuthenticatedGetFailed; (url: String); format!("failed to get the page at `{}'.", url)];
    [GettingTextFailed; (); "failed to get text from page.".to_string()];
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
            "needs_authenticate() is not implemetented for now, always returns `false'."
        );
        false
    }

    fn authenticate(&self, quiet: bool) -> result::Result<(), Box<dyn error::Error + Send>> {
        print_info!(
            !quiet,
            "authenticate() for AOJ is not implemented for now, do nothing."
        );
        Ok(())
    }

    fn fetch_test_case_files(
        &self,
        _quiet: bool,
    ) -> result::Result<Vec<TestCaseFile>, Box<dyn error::Error + Send>> {
        let text = download_text(self.problem.url())
            .chain(ErrorKind::FetchingProblemFailed(
                self.problem.problem_id().to_string(),
            ))
            .map_err(error_into_box)?;
        parse_text(text).map_err(error_into_box)
    }
}

fn error_into_box<T: 'static + error::Error + Send>(x: T) -> Box<dyn error::Error + Send> {
    Box::new(x)
}

pub fn parse_text(text: String) -> Result<Vec<TestCaseFile>> {
    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let mut pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 {
        return Err(Error::new(ErrorKind::UnexpectedNumberOfPreTag(pres.len())));
    }

    if pres.len() % 2 == 1 {
        pres = pres.into_iter().skip(1).collect();
    }

    let mut result = Vec::new();
    let beginning =
        TestCaseFile::next_unused_idx().chain(ErrorKind::CouldNotDetermineTestCaseFileName())?;
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
        .chain(ErrorKind::AuthenticatedGetFailed(url.to_string()))?
        .text()
        .chain(ErrorKind::GettingTextFailed())
}
