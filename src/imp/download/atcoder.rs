use super::{ContestProvider, Fetchers};
use crate::imp::auth::atcoder as auth;
use crate::imp::fetch::atcoder as fetch;
use crate::imp::fetch::atcoder::ATCODER_TOP;
use anyhow::ensure;
use anyhow::{Context, Result};
use easy_scraper::Pattern;
use itertools::Itertools;

pub struct AtCoder {
    contest: Contest,
}

impl AtCoder {
    pub fn new(contest: Contest) -> AtCoder {
        AtCoder { contest }
    }
}

pub enum Contest {
    ContestId { contest_id: String, url: String },
    DirectUrl { url: String },
}

impl Contest {
    pub fn from_contest_id(contest_id: String) -> Result<Contest> {
        ensure!(
            contest_id.len() == 6,
            "invalid format for contest id: `{}`",
            contest_id
        );
        let url = format!("{}/contests/{}/tasks", ATCODER_TOP, contest_id);

        Ok(Contest::ContestId { contest_id, url })
    }

    pub fn from_url(url: String) -> Contest {
        Contest::DirectUrl { url }
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

    fn make_fetchers(&self) -> Result<Fetchers> {
        let text = download_text(self.url())?;
        let fetchers = parse_table(&text)
            .into_iter()
            .map(|mut row| {
                let problem = fetch::Problem::from_url(format!("{}{}", ATCODER_TOP, row.url));
                let fetcher = Box::new(fetch::AtCoder::new(problem));

                row.problem.make_ascii_lowercase();
                super::Fetcher {
                    provider: fetcher as _,
                    problem_name: row.problem,
                }
            })
            .collect_vec();

        Ok(Fetchers {
            fetchers,
            contest_id: self.contest_id().to_string(),
            unique_contest_id: true,
        })
    }
}

fn download_text(url: &str) -> Result<String> {
    auth::authenticated_get(url)
        .with_context(|| format!("failed to fetch `{}` with logged in", url))?
        .text()
        .context("failed to get the html")
}

struct TableRow {
    problem: String,
    url: String,
}

fn parse_table(text: &str) -> Vec<TableRow> {
    let pat = Pattern::new(
        r#"<h2>Tasks</h2>
...
<div>
<table><tbody>
<tr>
<td class="text-center no-break">
<a href="{{url}}">{{problem}}</a>
</td>
<td>
<a href="{{url}}"
>{{name}}</a
>
</td>
</tr>
</tbody></table></div>"#,
    )
    .unwrap();

    pat.matches(text)
        .into_iter()
        .map(|r| TableRow {
            problem: r["problem"].clone(),
            url: r["url"].clone(),
        })
        .collect()
}
