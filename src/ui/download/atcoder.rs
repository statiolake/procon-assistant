use super::{ContestProvider, Fetchers};
use crate::imp::auth::atcoder as auth;
use crate::imp::fetch::atcoder as fetch;
use crate::imp::fetch::TestCaseProvider;
use crate::ui::login::atcoder as login;
use crate::ui::login::LoginUI;
use scraper::{Html, Selector};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("contest_id `{contest_id}` is invalid; the example format for AtCoder Grand Contest 022: agc022")]
    InvalidFormatForContestId { contest_id: String },

    #[error("failed to get contest page text")]
    GettingProblemPageFailed { source: anyhow::Error },

    #[error("failed to get tasks")]
    GettingTasksFailed,

    #[error("failed to get contest id")]
    GettingProblemIdFailed,

    #[error("contest id was empty")]
    EmptyProblemId,

    #[error("failed to get provider")]
    GettingProviderFailed { source: anyhow::Error },

    #[error("failed to get the page at `{url}`")]
    AuthenticatedGetFailed { source: anyhow::Error, url: String },

    #[error("failed to get text from page")]
    GettingTextFailed { source: anyhow::Error },
}

pub struct AtCoder {
    contest: Contest,
}

impl AtCoder {
    pub fn new(contest_id: String) -> Result<AtCoder> {
        Ok(AtCoder {
            contest: Contest::from_contest_id(contest_id)?,
        })
    }
}

pub enum Contest {
    ContestId { contest_id: String, url: String },
    DirectUrl { url: String },
}

impl Contest {
    pub fn from_contest_id(contest_id: String) -> Result<Contest> {
        let contest = if contest_id.starts_with("http") {
            Contest::DirectUrl { url: contest_id }
        } else {
            if contest_id.len() != 6 {
                return Err(Error::InvalidFormatForContestId { contest_id });
            }
            let url = format!("https://atcoder.jp/contests/{}/tasks", contest_id);
            Contest::ContestId { contest_id, url }
        };
        Ok(contest)
    }

    pub fn contest_id(&self) -> &str {
        match *self {
            Contest::ContestId { ref contest_id, .. } => &contest_id,
            Contest::DirectUrl { .. } => "Unknown",
        }
    }

    pub fn url(&self) -> &str {
        match *self {
            Contest::ContestId { ref url, .. } => url,
            Contest::DirectUrl { ref url } => url,
        }
    }
}

impl ContestProvider for AtCoder {
    fn site_name(&self) -> &str {
        "AtCoder"
    }

    fn contest_id(&self) -> &str {
        self.contest.contest_id()
    }

    fn url(&self) -> &str {
        self.contest.url()
    }

    fn make_fetchers(&self) -> anyhow::Result<Fetchers> {
        let id = self.contest.contest_id();
        let (beginning_char, numof_problems) = get_range_of_problems(id)?;

        let fetchers = (0..numof_problems).map(|problem| {
            let problem = (b'a' + problem) as char; // hack: atcoder regular contest starts 'a' while it's showed as 'c'
            let problem_id = format!("{}{}", self.contest.contest_id(), problem);
            fetcher_for(problem_id)
                .map(fetcher_into_box)
                .map(|t| (t, Box::new(login::AtCoder) as Box<dyn LoginUI>))
        });

        let fetchers: Result<Vec<_>> = fetchers.collect();
        let fetchers = fetchers?;

        let fetchers = super::Fetchers {
            contest_id: self.contest.contest_id().to_string(),
            fetchers,
            beginning_char,
        };

        Ok(fetchers)
    }
}

fn fetcher_into_box<T: 'static + TestCaseProvider>(x: T) -> Box<dyn TestCaseProvider> {
    Box::new(x)
}

// Result<(beginning_char, numof_problems)>
fn get_range_of_problems(contest_id: &str) -> Result<(char, u8)> {
    // fetch the tasks
    let url = format!("https://atcoder.jp/contests/{}/tasks", contest_id);
    let text =
        download_text(&url).map_err(|e| Error::GettingProblemPageFailed { source: e.into() })?;

    let document = Html::parse_document(&text);
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_tr = Selector::parse("tr").unwrap();
    let sel_a = Selector::parse("a").unwrap();

    // get rows in table
    let rows: Vec<_> = document
        .select(&sel_tbody)
        .next()
        .ok_or_else(|| Error::GettingTasksFailed)?
        .select(&sel_tr)
        .collect();

    let numof_problems = rows.len() as u8;
    let beginning_char_uppercase = rows[0]
        .select(&sel_a)
        .next()
        .ok_or_else(|| Error::GettingProblemIdFailed)?
        .inner_html()
        .chars()
        .next()
        .ok_or_else(|| Error::EmptyProblemId)?;

    Ok((
        beginning_char_uppercase.to_lowercase().next().unwrap(),
        numof_problems,
    ))
}

fn fetcher_for(problem_id: String) -> Result<fetch::AtCoder> {
    fetch::AtCoder::new(problem_id).map_err(|e| Error::GettingProviderFailed { source: e.into() })
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
