use reqwest;
use scraper::{Html, Selector};

use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;

use super::print_msg;
use fetch::atcoder;
use initdirs;

fn print_contest_name_err() {
    print_error!("contest_name's format is invalid. the example format for AtCoder Grand Contest 022: agc022");
}

const CONTEST: &str = "AtCoder";
const CONTEST_TYPES: [&str; 3] = ["abc", "arc", "agc"];

pub fn main(contest_name: &str) -> bool {
    if contest_name.len() != 6 {
        print_contest_name_err();
        return false;
    }

    let contest_type = &contest_name[0..3];
    let round = &contest_name[3..6];
    if !CONTEST_TYPES.contains(&contest_type) || round.parse::<i32>().is_err() {
        print_contest_name_err();
        return false;
    }

    let (beginning, numof) = match get_range_of_problems(contest_name) {
        Ok(t) => t,
        Err(e) => {
            print_msg::err_in_fetching_tasks(CONTEST, &format!("{:?}", e));
            return false;
        }
    };

    initdirs::create_directories(contest_name, beginning, numof);
    env::set_current_dir(&Path::new(contest_name)).unwrap();
    let mut id = String::from(contest_name);
    for i in 0..numof {
        let curr = (beginning as u8 + i as u8) as char;
        env::set_current_dir(&Path::new(&curr.to_string())).unwrap();
        id.push(curr);
        atcoder::main(&id);
        id.pop();
        env::set_current_dir(&Path::new("..")).unwrap();
    }

    return true;
}

// temporary!!
#[derive(Debug)]
struct SomeError {
    description: String,
}

impl fmt::Display for SomeError {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{:?}", self)
    }
}

impl SomeError {
    pub fn new(description: String) -> SomeError {
        SomeError { description }
    }
}

impl Error for SomeError {
    fn cause(&self) -> Option<&Error> {
        None
    }

    fn description(&self) -> &str {
        &self.description
    }
}

// Result<(beginning_char, numof_problems), error>
fn get_range_of_problems(contest_name: &str) -> Result<(char, u8), Box<Error>> {
    // fetch the tasks
    let url = format!("https://beta.atcoder.jp/contests/{}/tasks", contest_name);
    print_msg::in_fetching_tasks(CONTEST);
    let text = reqwest::get(&url)?.text()?;

    let document = Html::parse_document(&text);
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_tr = Selector::parse("tr").unwrap();
    let sel_a = Selector::parse("a").unwrap();

    // get rows in table
    let rows = document
        .select(&sel_tbody)
        .next()
        .map(|p| p.select(&sel_tr).collect::<Vec<_>>())
        .ok_or(SomeError::new("failed to get the tasks.".into()))?;

    let numof = rows.len() as u8;
    let beginning_char_uppercase = rows[0]
        .select(&sel_a)
        .next()
        .ok_or(SomeError::new("failed to get the problem id.".into()))?
        .inner_html()
        .chars()
        .next()
        .ok_or(SomeError::new("the problem id is empty string".into()))?;

    Ok((
        beginning_char_uppercase.to_lowercase().next().unwrap(),
        numof,
    ))
}
