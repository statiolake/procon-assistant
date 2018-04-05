use reqwest;
use scraper::{Html, Selector};

use addcase;
use common;

use super::print_msg;

use Error;
use Result;

const CONTEST: &str = "Aizu Online Judge";

pub fn main(problem_id: &str) -> Result<()> {
    let text = download_text(problem_id).map_err(|e| {
        Some(Error::new(
            "downloading html",
            format!("failed to fetch the problem {}", problem_id),
            Some(Box::new(e)),
        ))
    })?;

    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Some(Error::new(
            "parsing problem html",
            format!(
                "the number of <pre> elements is unexpected: detected {}",
                pres.len()
            ),
            None,
        )));
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(CONTEST, problem_id, i + 1);
        let (infile_name, outfile_name) = common::make_next_iofile_name().map_err(|e| {
            Some(Error::new(
                "creating testcase file",
                "failed to generate testcase file's name.",
                Some(Box::new(e.unwrap())),
            ))
        })?;

        addcase::ensure_create(&infile_name, &pres[i * 2 + 1].inner_html())?;
        addcase::ensure_create(&outfile_name, &pres[i * 2 + 2].inner_html())?;
    }

    print_msg::in_generating_sample_case_finished(CONTEST, problem_id, pres.len() / 2);

    Ok(())
}

fn download_text(id: &str) -> reqwest::Result<String> {
    let url = format!(
        "http://judge.u-aizu.ac.jp/onlinejudge/description.jsp?lang=jp&id={}",
        id,
    );
    print_msg::in_fetching_problem(CONTEST, id, &url);
    reqwest::get(&url)?.text()
}
