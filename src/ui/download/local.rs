use super::Fetchers;
use crate::ui::fetch;
use crate::ui::fetch::ProblemDescriptor;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::result;
use std::str;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("problems.txt not found in this directory")]
    AnythingNotSpecified,

    #[error("couldn't open `{file_path}`")]
    CouldNotOpenProblemsTxt {
        source: anyhow::Error,
        file_path: String,
    },

    #[error("couldn't read `{file_path}`")]
    CouldNotReadProblemsTxt {
        source: anyhow::Error,
        file_path: String,
    },

    #[error("failed to parse specified problem")]
    ParseFailed { source: anyhow::Error },
}

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

    fn make_fetchers(&self, _quiet: bool) -> result::Result<Fetchers, anyhow::Error> {
        let problem_list = load_problem_list(self.file_path.clone())?;
        make_fetcher(problem_list).map_err(Into::into)
    }
}

fn make_fetcher(problem_list: Vec<String>) -> Result<Fetchers> {
    problem_list
        .into_iter()
        .map(|problem| {
            ProblemDescriptor::parse(problem).map_err(|e| Error::ParseFailed { source: e.into() })
        })
        .map(|pd| {
            pd.and_then(|pd| {
                fetch::get_provider(pd).map_err(|e| Error::ParseFailed { source: e.into() })
            })
        })
        .collect::<Result<_>>()
        .map(|fetchers| Fetchers {
            fetchers,
            contest_id: ".".to_string(),
            beginning_char: 'a',
        })
}

fn load_problem_list(file_path: String) -> Result<Vec<String>> {
    let problems_path = Path::new(&file_path);
    if !problems_path.exists() {
        return Err(Error::AnythingNotSpecified);
    }
    let mut f = File::open(problems_path).map_err(|e| Error::CouldNotOpenProblemsTxt {
        source: e.into(),
        file_path: file_path.clone(),
    })?;
    let mut content = String::new();
    f.read_to_string(&mut content)
        .map_err(|e| Error::CouldNotReadProblemsTxt {
            source: e.into(),
            file_path: file_path.clone(),
        })?;
    let res: Vec<_> = content
        .split('\n')
        .map(str::trim)
        .filter(|&x| !x.starts_with('#'))
        .filter(|&x| x != "")
        .map(|x| x.into())
        .collect();

    Ok(res)
}
