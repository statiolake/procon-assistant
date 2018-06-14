use std::error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::result;
use std::str;

use super::Fetchers;
use fetch;
use fetch::ProblemDescriptor;

define_error!();
define_error_kind! {
    [AnythingNotSpecified; (); format!("problems.txt not found in this directory.")];
    [CouldNotOpenProblemsTxt; (file_path: String); format!("couldn't open `{}'.", file_path)];
    [CouldNotReadProblemsTxt; (file_path: String); format!("couldn't read `{}'.", file_path)];
    [ParseFailed; (); format!("failed to parse specified problem.")];
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

    fn make_fetchers(&self) -> result::Result<Fetchers, Box<dyn error::Error + Send>> {
        let problem_list =
            load_problem_list(self.file_path.clone()).map_err(|e| (box e) as Box<_>)?;
        make_fetcher(problem_list).map_err(|e| (box e) as Box<_>)
    }
}

fn make_fetcher(problem_list: Vec<String>) -> Result<Fetchers> {
    problem_list
        .into_iter()
        .map(|problem| ProblemDescriptor::parse(problem).chain(ErrorKind::ParseFailed()))
        .map(|pd| pd.and_then(|pd| fetch::get_provider(pd).chain(ErrorKind::ParseFailed())))
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
        return Err(Error::new(ErrorKind::AnythingNotSpecified()));
    }
    let mut f =
        File::open(problems_path).chain(ErrorKind::CouldNotOpenProblemsTxt(file_path.clone()))?;
    let mut content = String::new();
    f.read_to_string(&mut content)
        .chain(ErrorKind::CouldNotReadProblemsTxt(file_path.clone()))?;
    let res: Vec<_> = content
        .split('\n')
        .map(|x| x.trim())
        .filter(|&x| !x.starts_with("#"))
        .filter(|&x| x != "")
        .map(|x| x.into())
        .collect();

    Ok(res)
}
