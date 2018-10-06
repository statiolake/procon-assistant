pub mod atcoder;
pub mod local;

use std::env;
use std::error;
use std::path::Path;
use std::result;
use std::str;

use self::atcoder::AtCoder;
use self::local::Local;
use fetch;
use fetch::TestCaseProvider;
use initdirs;
use tags::SPACER;

define_error!();
define_error_kind! {
    [ArgumentFormatError; (passed_arg: String); format!(concat!(
        "argument's format is not collect: `{}'.\n",
        "{}please specify contest-site and problem-id separated by `:' (colon)."
    ), passed_arg, SPACER)];
    [UnknownContestSite; (site: String); format!("contest-site `{}' is unknown.", site)];
    [MakingFetcherFailed; (); "failed to make fetchers.".to_string()];
    [FetchError; (); "failed to fetch the problem".to_string()];
    [ProviderCreationFailed; (); "failed to create provider.".to_string()];
    [WritingTestCaseFailed; (); "failed to write test case file.".to_string()];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let provider = match args.into_iter().next() {
        Some(arg) => get_provider(arg),
        None => handle_empty_arg(),
    }?;

    let fetchers = provider
        .make_fetchers()
        .map_err(|e| Error::with_cause(ErrorKind::MakingFetcherFailed(), e))?;

    print_fetching!("{} (at {})", provider.contest_id(), provider.url());
    fetchers.prepare_generate();
    for (problem, fetcher) in fetchers.fetchers.into_iter().enumerate() {
        generate_one(
            fetchers.contest_id.clone(),
            fetchers.beginning_char,
            problem as u8,
            fetcher,
        )?;
    }

    Ok(())
}

fn get_provider(arg: String) -> Result<Box<dyn ContestProvider>> {
    let (contest_site, contest_id) = parse_arg(&arg)?;
    match contest_site {
        "atcoder" | "at" => {
            AtCoder::new(contest_id.to_string()).chain(ErrorKind::ProviderCreationFailed())
        }
        _ => Err(Error::new(ErrorKind::UnknownContestSite(
            contest_site.to_string(),
        ))),
    }
    .map(|provider| (box provider) as Box<_>)
}

fn get_local_provider() -> Result<Box<dyn ContestProvider>> {
    Ok((box Local::from_path("problems.txt".to_string())) as Box<_>)
}

fn parse_arg(arg: &str) -> Result<(&str, &str)> {
    let sp: Vec<_> = arg.split(':').collect();
    if sp.len() != 2 {
        return Err(Error::new(ErrorKind::ArgumentFormatError(arg.to_string())));
    }
    Ok((sp[0], sp[1]))
}

fn handle_empty_arg() -> Result<Box<dyn ContestProvider>> {
    Ok(env::current_dir()
        .map_err(|_| ())
        .and_then(|current_dir| {
            current_dir
                .file_name()
                .ok_or(())
                .and_then(|file_name| file_name.to_str().ok_or(()))
                .map(|file_name| file_name.to_string())
        })
        .map(|file_name| file_name.to_string())
        .and_then(|file_name| {
            if ["abc", "arc", "agc"].contains(&&file_name[0..3]) {
                AtCoder::new(file_name)
                    .map(|p| (box p) as Box<_>)
                    .map_err(|_| ())
            } else {
                Err(())
            }
        })
        .unwrap_or(get_local_provider()?))
}

pub struct Fetchers {
    fetchers: Vec<Box<dyn TestCaseProvider>>,
    contest_id: String,
    beginning_char: char,
}

impl Fetchers {
    pub fn prepare_generate(&self) {
        let numof_problems = self.fetchers.len() as u8;
        change_current_dir(&self.contest_id, self.beginning_char, numof_problems);
    }
}

pub fn generate_one(
    mut contest_id: String,
    beginning_char: char,
    problem: u8,
    fetcher: Box<dyn TestCaseProvider + 'static>,
) -> Result<()> {
    let curr_actual = (beginning_char as u8 + problem) as char;
    env::set_current_dir(Path::new(&curr_actual.to_string())).unwrap();

    let curr_url = ('a' as u8 + problem) as char;
    contest_id.push(curr_url);
    let tcfs = fetch::fetch_test_case_files(fetcher).chain(ErrorKind::FetchError())?;
    fetch::write_test_case_files(tcfs).chain(ErrorKind::WritingTestCaseFailed())?;
    contest_id.pop();

    env::set_current_dir(Path::new("..")).unwrap();

    Ok(())
}

fn change_current_dir(contest_id: &str, beginning_char: char, numof_problems: u8) {
    let current_dir = env::current_dir().unwrap();
    let file_name = current_dir.file_name();
    let executed_inside_proper_dir = file_name.is_some() && file_name.unwrap() == contest_id;
    if executed_inside_proper_dir {
        env::set_current_dir("..").unwrap();
    }
    initdirs::create_directories(contest_id, beginning_char, numof_problems);
    env::set_current_dir(&Path::new(contest_id)).unwrap();
}

pub trait ContestProvider {
    fn site_name(&self) -> &str;
    fn contest_id(&self) -> &str;
    fn url(&self) -> &str;

    fn make_fetchers(&self) -> result::Result<Fetchers, Box<dyn error::Error + Send>>;
}
