use reqwest;
use scraper::{Html, Selector};

use addcase;
use common;

use super::print_msg;

const CONTEST: &str = "AtCoder";
const CONTEST_TYPES: [&str; 3] = ["abc", "arc", "agc"];

fn print_id_err() {
    print_error!("id's format is invalid. the example format for AtCoder Grand Contest 022 Problem A: agc022a");
}

pub fn main(id: &str) -> bool {
    if id.len() != 7 {
        print_id_err();
        return false;
    }

    let contest_type = &id[0..3];
    let round = &id[3..6];
    let contest_name = &id[0..6];
    let problem = &id[6..7];
    if !CONTEST_TYPES.contains(&contest_type) || round.parse::<i32>().is_err() {
        print_id_err();
        return false;
    }

    let text = match download_text(id, contest_name, problem) {
        Ok(text) => text,
        Err(e) => {
            print_msg::err_in_fetching_problem(CONTEST, id, &format!("{:?}", e));
            return false;
        }
    };
    let document = Html::parse_document(&text);
    let sel_div_task_statement = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let pres = match document.select(&sel_div_task_statement).next() {
        Some(p) => p,
        None => {
            print_msg::err_in_parsing_problem(
                CONTEST,
                id,
                &format!("failed to get div#task-statement."),
            );
            return false;
        }
    };
    let pres = match pres.select(&sel_span_ja).next() {
        Some(p) => p,
        None => {
            print_msg::err_in_parsing_problem(CONTEST, id, &format!("failed to get span.lang-ja."));
            return false;
        }
    };
    let pres: Vec<_> = pres.select(&sel_pre).collect();
    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        print_msg::err_in_parsing_problem(
            CONTEST,
            id,
            &format!("the number of <pre> elements is unexpected: {}", pres.len()),
        );
        return false;
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(CONTEST, id, i + 1);
        let (infile_name, outfile_name) = match common::make_next_iofile_name() {
            Ok(r) => r,
            Err(_) => return false,
        };
        addcase::ensure_create(&infile_name, &pres[i * 2 + 1].inner_html());
        addcase::ensure_create(&outfile_name, &pres[i * 2 + 2].inner_html());
    }

    print_msg::in_generating_sample_case_finished(CONTEST, id, pres.len() / 2);

    true
}

fn download_text(id: &str, contest_name: &str, problem: &str) -> reqwest::Result<String> {
    let url = format!(
        "https://beta.atcoder.jp/contests/{}/tasks/{}_{}",
        contest_name, contest_name, problem
    );
    print_msg::in_fetching_problem(CONTEST, id, &url);
    reqwest::get(&url)?.text()
}
