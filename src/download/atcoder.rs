use scraper::{Html, Selector};

use std::error;
use std::result;

use fetch::atcoder::AtCoder as FetchAtCoder;
use fetch::TestCaseProvider;
use imp::auth::atcoder as auth;

define_error!();
define_error_kind!{
    [InvalidFormatForContestId; (contest_id: String); format!(
        "contest_id `{}' is invalid; the example format for AtCoder Grand Contest 022: agc022",
        contest_id
    )];
    [GettingProblemPageFailed; (); "failed to get contest page text.".to_string()];
    [GettingTasksFailed; (); "failed to get tasks.".to_string()];
    [GettingProblemIdFailed; (); "failed to get contest id.".to_string()];
    [EmptyProblemId; (); "contest id was empty.".to_string()];
    [GettingProviderFailed; (); format!("failed to get provider.")];
    [AuthenticatedGetFailed; (url: String); format!("failed to get the page at `{}'.", url)];
    [GettingTextFailed; (); format!("failed to get text from page.")];
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
                return Err(Error::new(ErrorKind::InvalidFormatForContestId(contest_id)));
            }
            let url = format!("https://beta.atcoder.jp/contests/{}/tasks", contest_id);
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

impl super::ContestProvider for AtCoder {
    fn site_name(&self) -> &str {
        "AtCoder"
    }

    fn contest_id(&self) -> &str {
        self.contest.contest_id()
    }

    fn url(&self) -> &str {
        self.contest.url()
    }

    fn make_fetchers(&self) -> result::Result<super::Fetchers, Box<dyn error::Error + Send>> {
        let (beginning_char, numof_problems) =
            get_range_of_problems(self.contest.contest_id()).map_err(|e| (box e) as Box<_>)?;
        (0..numof_problems)
            .into_iter()
            .map(|problem| {
                let problem = ('a' as u8 + problem) as char; // hack: atcoder regular contest starts 'a' while it's showed as 'c'
                let problem_id = format!("{}{}", self.contest.contest_id(), problem);
                fetcher_for(problem_id).map(|fetcher| (box fetcher) as Box<dyn TestCaseProvider>)
            })
            .collect::<Result<Vec<Box<dyn TestCaseProvider>>>>()
            .map(|fetchers| super::Fetchers {
                contest_id: self.contest.contest_id().to_string(),
                fetchers,
                beginning_char,
            })
            .map_err(|e| (box e) as Box<_>)
    }
}

// Result<(beginning_char, numof_problems)>
fn get_range_of_problems(contest_id: &str) -> Result<(char, u8)> {
    // fetch the tasks
    let url = format!("https://beta.atcoder.jp/contests/{}/tasks", contest_id);
    let text = download_text(&url).chain(ErrorKind::GettingProblemPageFailed())?;

    let document = Html::parse_document(&text);
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_tr = Selector::parse("tr").unwrap();
    let sel_a = Selector::parse("a").unwrap();

    // get rows in table
    let rows: Vec<_> = document
        .select(&sel_tbody)
        .next()
        .ok_or(Error::new(ErrorKind::GettingTasksFailed()))?
        .select(&sel_tr)
        .collect();

    let numof_problems = rows.len() as u8;
    let beginning_char_uppercase = rows[0]
        .select(&sel_a)
        .next()
        .ok_or(Error::new(ErrorKind::GettingProblemIdFailed()))?
        .inner_html()
        .chars()
        .next()
        .ok_or(Error::new(ErrorKind::EmptyProblemId()))?;

    Ok((
        beginning_char_uppercase.to_lowercase().next().unwrap(),
        numof_problems,
    ))
}

fn fetcher_for(problem_id: String) -> Result<FetchAtCoder> {
    FetchAtCoder::new(problem_id).chain(ErrorKind::GettingProviderFailed())
}

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .chain(ErrorKind::AuthenticatedGetFailed(url.to_string()))?
        .text()
        .chain(ErrorKind::GettingTextFailed())
}
