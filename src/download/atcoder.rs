use reqwest;
use scraper::{Html, Selector};

use std::env;
use std::path::Path;

use super::print_msg;
use fetch;
use imp::auth;
use initdirs;

define_error!();
define_error_kind!{
    [UnknownContestName; (contest_name: String); format!(
        "unknown contest name: `{}'", contest_name
    )];
    [InvalidContestId; (contest_id: String); format!(
        "contest_id `{}' is invalid; the example format for AtCoder Grand Contest 022: agc022",
        contest_id
    )];
    [GettingProblemPageFailed; (); "failed to get problem page text.".to_string()];
    [GettingTasksFailed; (); "failed to get tasks.".to_string()];
    [GettingProblemIdFailed; (); "failed to get problem id.".to_string()];
    [EmptyProblemId; (); "problem id was empty.".to_string()];
    [ChildError; (); format!("during processing")];
}

fn get_long_contest_name(contest_name: &str) -> Result<&str> {
    match contest_name {
        "abc" => Ok("AtCoder Beginner Contest"),
        "arc" => Ok("AtCoder Regular Contest"),
        "agc" => Ok("AtCoder Grand Contest"),
        _ => Err(Error::new(ErrorKind::UnknownContestName(
            contest_name.to_string(),
        ))),
    }
}

pub fn main(contest_id: &str) -> Result<()> {
    if contest_id.len() != 6 {
        return Err(Error::new(ErrorKind::InvalidContestId(
            contest_id.to_string(),
        )));
    }

    let contest_name = &contest_id[0..3];
    let long_contest_name = get_long_contest_name(contest_name)?;
    // let round = &contest_id[3..6];

    let (beginning_char, numof_problems) = get_range_of_problems(long_contest_name, contest_id)?;

    let current_dir = env::current_dir().unwrap();
    let file_name = current_dir.file_name();
    let executed_inside_proper_dir = file_name.is_some() && file_name.unwrap() == contest_id;
    if executed_inside_proper_dir {
        env::set_current_dir("..").unwrap();
    }

    initdirs::create_directories(contest_id, beginning_char, numof_problems);

    env::set_current_dir(&Path::new(contest_id)).unwrap();
    let mut problem_id = String::from(contest_id);
    for problem in 0..numof_problems {
        let curr_actual = (beginning_char as u8 + problem) as char;
        env::set_current_dir(Path::new(&curr_actual.to_string())).unwrap();

        let curr_url = ('a' as u8 + problem) as char;
        problem_id.push(curr_url);
        fetch::atcoder::main(&problem_id).chain(ErrorKind::ChildError())?;
        problem_id.pop();

        env::set_current_dir(Path::new("..")).unwrap();
    }

    Ok(())
}

// Result<(beginning_char, numof_problems)>
fn get_range_of_problems(long_contest_name: &str, contest_id: &str) -> Result<(char, u8)> {
    // fetch the tasks
    let url = format!("https://beta.atcoder.jp/contests/{}/tasks", contest_id);
    print_msg::in_fetching_tasks(long_contest_name);
    let text = download_text_by_url(&url).chain(ErrorKind::GettingProblemPageFailed())?;

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

fn download_text_by_url(url: &str) -> reqwest::Result<String> {
    let mut res = auth::atcoder::get_with_auth(url)?;
    auth::atcoder::store_revel_session_from_response(&mut res, false).ok();
    res.text()
}
