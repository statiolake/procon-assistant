use reqwest;
use scraper::{Html, Selector};

use std::env;
use std::path::Path;

use super::print_msg;
use fetch::atcoder;
use initdirs;

use Error;
use Result;

fn get_long_contest_name(contest_name: &str) -> Result<&str> {
    let conversion_error = Error::new(
        "converting short name to long name",
        format!("unknown contest name {}", contest_name),
    );

    match contest_name {
        "abc" => Ok("AtCoder Beginner Contest"),
        "arc" => Ok("AtCoder Regular Contest"),
        "agc" => Ok("AtCoder Grand Contest"),
        _ => Err(conversion_error),
    }
}

pub fn main(contest_id: &str) -> Result<()> {
    let contest_id_error = Error::new(
        "parsing contest_id",
        "format is invalid; the example format for AtCoder Grand Contest 022: agc022",
    );

    if contest_id.len() != 6 {
        return Err(contest_id_error);
    }

    let contest_name = &contest_id[0..3];
    let long_contest_name = get_long_contest_name(contest_name)?;
    // let round = &contest_id[3..6];

    let (beginning_char, numof_problems) = get_range_of_problems(long_contest_name, contest_id)?;

    initdirs::create_directories(contest_id, beginning_char, numof_problems);

    env::set_current_dir(&Path::new(contest_id)).unwrap();
    let mut problem_id = String::from(contest_id);
    for problem in 0..numof_problems {
        let curr = (beginning_char as u8 + problem) as char;
        env::set_current_dir(Path::new(&curr.to_string())).unwrap();
        problem_id.push(curr);
        atcoder::main(&problem_id)?;
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
    let text = reqwest::get(&url)
        .and_then(|mut g| g.text())
        .map_err(|e| Error::with_cause("downloading html", "failed to get text", box e))?;

    let document = Html::parse_document(&text);
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_tr = Selector::parse("tr").unwrap();
    let sel_a = Selector::parse("a").unwrap();

    // get rows in table
    let rows: Vec<_> = document
        .select(&sel_tbody)
        .next()
        .ok_or(Error::new("parsing html", "failed to get the tasks."))?
        .select(&sel_tr)
        .collect();

    let numof_problems = rows.len() as u8;
    let beginning_char_uppercase = rows[0]
        .select(&sel_a)
        .next()
        .ok_or(Error::new("parsing html", "failed to get the problem id."))?
        .inner_html()
        .chars()
        .next()
        .ok_or(Error::new("parsing html", "the problem id is empty string"))?;

    Ok((
        beginning_char_uppercase.to_lowercase().next().unwrap(),
        numof_problems,
    ))
}
