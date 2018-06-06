use std::error;
use std::result;

use scraper::{Html, Selector};

use imp::auth::atcoder as auth;
use imp::test_case::TestCaseFile;
use login::atcoder as login;

define_error!();
define_error_kind! {
    [FindingTagFailed; (selector: String); format!("missing tag: failed to find `{}'\nmaybe you are not logged in?", selector)];
    [UnexpectedNumberOfPreTag; (detected: usize); format!("unexpected number of <pre>: {}", detected)];
    [CouldNotDetermineTestCaseFileName; (); format!("failed to determine test case file name.")];
    [AuthenticatedGetFailed; (url: String); format!("failed to get the page at `{}'.", url)];
    [GettingTextFailed; (); format!("failed to get text from page.")];
    [InvalidFormatForProblemId; (problem: String); format!(concat!(
        "invalid format for problem-id: `{}'\n",
        "example: `abc022a' for AtCoder Beginner Contest 022 Problem A"
    ), problem)];
    [LoginFailed; (); format!("logging in failed.")];
}

pub struct AtCoder {
    problem: Problem,
}

impl AtCoder {
    pub fn new(problem_id: String) -> Result<AtCoder> {
        Problem::from_problem_id(problem_id).map(|problem| AtCoder { problem })
    }
}

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
                return Err(Error::new(ErrorKind::InvalidFormatForProblemId(problem_id)));
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

    fn needs_authenticate(&self) -> bool {
        print_info!(
            true,
            "needs_authenticate() is not implemetented for now, always returns `false'."
        );
        false
    }

    fn authenticate(&self) -> result::Result<(), Box<dyn error::Error + Send>> {
        login::main()
            .chain(ErrorKind::LoginFailed())
            .map_err(|e| (box e) as Box<_>)
    }

    fn fetch_test_case_files(
        &self,
    ) -> result::Result<Vec<TestCaseFile>, Box<dyn error::Error + Send>> {
        let text = download_text(self.problem.url()).map_err(|e| (box e) as Box<_>)?;
        parse_text(text).map_err(|e| (box e) as Box<_>)
    }
}

fn parse_text(text: String) -> Result<Vec<TestCaseFile>> {
    let document = Html::parse_document(&text);
    let sel_div_task_statement = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document
        .select(&sel_div_task_statement)
        .next()
        .ok_or(Error::new(ErrorKind::FindingTagFailed(
            "div#task-statement".into(),
        )))?
        .select(&sel_span_ja)
        .next()
        .ok_or(Error::new(ErrorKind::FindingTagFailed(
            "div#span.lang-ja".into(),
        )))?
        .select(&sel_pre)
        .collect();

    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Error::new(ErrorKind::UnexpectedNumberOfPreTag(pres.len())));
    }

    let beginning =
        TestCaseFile::next_unused_idx().chain(ErrorKind::CouldNotDetermineTestCaseFileName())?;
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

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .chain(ErrorKind::AuthenticatedGetFailed(url.to_string()))?
        .text()
        .chain(ErrorKind::GettingTextFailed())
}
