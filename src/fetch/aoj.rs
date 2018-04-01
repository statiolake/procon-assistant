use request;
use scraper;
use scraper::{Html, Selector};

use addcase;
use common;

pub fn main(id: &str) -> bool {
    let text = match download_text(id) {
        Ok(text) => text,
        Err(e) => {
            eprintln!(
                "failed to download problem of id {} in Aizu Online Judge.",
                id
            );
            return false;
        }
    };
    let document = Html::parse_document(text);
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        eprintln!("failed to parse problem of id {} in Aizu Online Judge", id);
        return false;
    }

    for i in 0..(pres.len() / 2) {
        let (infile_name, outfile_name) = match common::make_next_iofile_name() {
            Ok(r) => r,
            Err(_) => return false,
        };
        addcase::ensure_create(&infile_name, pres[i * 2 + 1].inner_html());
        addcase::ensure_create(&outfile_name, pres[i * 2 + 2].inner_html());
    }

    true
}

fn download_text(id: &str) -> String {
    let url = format(
        "http://judge.u-aizu.ac.jp/onlinejudge/description.jsp?lang=jp&id={}",
        id,
    );
    request::get(url)?.text()?;
}
