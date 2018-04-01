use reqwest;
use scraper::{Html, Selector};

use addcase;
use common;

pub fn main(id: &str) -> bool {
    let text = match download_text(id) {
        Ok(text) => text,
        Err(e) => {
            eprintln!(
                "failed to download problem of id {} in Aizu Online Judge due to {:?}.",
                id, e
            );
            return false;
        }
    };
    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        eprintln!("failed to parse problem of id {} in Aizu Online Judge", id);
        return false;
    }

    for i in 0..(pres.len() / 2) {
        print_generating!(
            "Sample Case {} in Aizu Online Judge Problem ID: {}",
            i + 1,
            id
        );
        let (infile_name, outfile_name) = match common::make_next_iofile_name() {
            Ok(r) => r,
            Err(_) => return false,
        };
        addcase::ensure_create(&infile_name, &pres[i * 2 + 1].inner_html());
        addcase::ensure_create(&outfile_name, &pres[i * 2 + 2].inner_html());
    }

    print_finished!(
        "Generating Test Cases in Aizu Online Judge Problem ID: {}",
        id
    );

    true
}

fn download_text(id: &str) -> reqwest::Result<String> {
    print_fetching!("Aizu Online Judge Problem ID: {}", id);
    let url = format!(
        "http://judge.u-aizu.ac.jp/onlinejudge/description.jsp?lang=jp&id={}",
        id,
    );
    reqwest::get(&url)?.text()
}
