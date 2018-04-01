use reqwest;
use scraper::{Html, Selector};

use addcase;
use common;

use super::print_msg;

const CONTEST: &str = "Aizu Online Judge";

pub fn main(id: &str) -> bool {
    let text = match download_text(id) {
        Ok(text) => text,
        Err(e) => {
            print_msg::err_in_fetching_problem(CONTEST, id, &format!("{:?}", e));
            return false;
        }
    };
    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document.select(&sel_pre).collect();
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

fn download_text(id: &str) -> reqwest::Result<String> {
    let url = format!(
        "http://judge.u-aizu.ac.jp/onlinejudge/description.jsp?lang=jp&id={}",
        id,
    );
    print_msg::in_fetching_problem(CONTEST, id, &url);
    reqwest::get(&url)?.text()
}
