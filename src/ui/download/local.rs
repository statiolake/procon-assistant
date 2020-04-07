use super::Fetchers;
use crate::imp::fetch::ProblemDescriptor;
use crate::ui::fetch;
use anyhow::ensure;
use anyhow::{Context, Result};
use itertools::Itertools;
use std::fs;
use std::path::Path;
use std::str;

pub struct Local {
    file_path: String,
}

impl Local {
    pub fn from_path(file_path: String) -> Local {
        Local { file_path }
    }
}

impl super::ContestProvider for Local {
    fn site_name(&self) -> &str {
        "Local"
    }

    fn contest_id(&self) -> &str {
        "."
    }

    fn url(&self) -> &str {
        &self.file_path
    }

    fn make_fetchers(&self) -> Result<Fetchers> {
        let problem_list = load_problem_list(self.file_path.clone())?;
        make_fetcher(problem_list).map_err(Into::into)
    }
}

fn make_fetcher(problem_list: Vec<String>) -> Result<Fetchers> {
    ensure!(
        problem_list.len() <= 26,
        "too many problems; max is 26 (a - z) but listed {} problems",
        problem_list.len()
    );
    let pds: Vec<_> = problem_list
        .into_iter()
        .map(|problem| ProblemDescriptor::parse(problem).context("failed to parse a problem"))
        .map(|pd| pd.and_then(|pd| fetch::get_provider(pd).context("failed to get the provider")))
        .collect::<Result<_>>()?;

    let fetchers = pds
        .into_iter()
        .enumerate()
        .map(|(idx, (fetcher, login_ui))| super::Fetcher {
            fetcher,
            login_ui,
            problem: (b'a'
                .checked_add(idx as u8)
                .expect("internal error: this must not overflow as it's checked before.")
                as char)
                .to_string(),
        })
        .collect_vec();

    Ok(Fetchers {
        fetchers,
        contest_id: "local".to_string(),
    })
}

fn load_problem_list(file_path: String) -> Result<Vec<String>> {
    let problems_path = Path::new(&file_path);
    ensure!(problems_path.exists(), "problem list is not specified");
    let content = fs::read_to_string(problems_path)
        .with_context(|| format!("failed to open problems: `{}`", problems_path.display()))?;
    let res = content
        .split('\n')
        .map(str::trim)
        .filter(|&x| !x.starts_with('#'))
        .filter(|&x| x != "")
        .map(Into::into)
        .collect_vec();

    Ok(res)
}
