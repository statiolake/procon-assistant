use reqwest;
use scraper::{Html, Selector};

use addcase::ensure_create;
use common;

use super::print_msg;

use Error;
use Result;

// const CONTEST: &str = "AtCoder";
fn get_long_contest_name(contest_name: &str) -> Result<&str> {
    let conversion_error = Error::new(
        "converting short name to long name",
        format!("unknown contest name {}", contest_name),
        None,
    );

    match contest_name {
        "abc" => Ok("AtCoder Beginner Contest"),
        "arc" => Ok("AtCoder Regular Contest"),
        "agc" => Ok("AtCoder Grand Contest"),
        _ => Err(Some(conversion_error)),
    }
}

pub fn main(problem_id: &str) -> Result<()> {
    if problem_id.len() != 7 {
        let problem_id_error = Error::new(
            "parsing problem_id",
            "format is invalid; the example format for AtCoder Grand Contest 022 Problem A: agc022a",
            None,
        );

        return Err(Some(problem_id_error));
    }

    let contest_name = &problem_id[0..3];
    let long_contest_name = get_long_contest_name(contest_name)?;
    // let round = &problem_id[3..6];
    let contest_id = &problem_id[0..6];
    let problem = &problem_id[6..7];

    let text = download_text(long_contest_name, problem_id, contest_id, problem).map_err(|e| {
        Some(Error::new(
            "downloading html",
            format!(
                "failed to fetch the problem {} problem {}",
                long_contest_name, problem
            ),
            Some(Box::new(e)),
        ))
    })?;

    let document = Html::parse_document(&text);
    let sel_div_task_statement = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let div_task_statement_not_found = Error::new(
        "parsing problem html",
        "failed to get div#task-statement",
        None,
    );

    let span_lang_ja_not_found = Error::new(
        "parsing problem html",
        "failed to get div#span.lang-ja",
        None,
    );
    let pres: Vec<_> = document
        .select(&sel_div_task_statement)
        .next()
        .ok_or(Some(div_task_statement_not_found))?
        .select(&sel_span_ja)
        .next()
        .ok_or(Some(span_lang_ja_not_found))?
        .select(&sel_pre)
        .collect();

    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Some(Error::new(
            "parsing problem html",
            format!(
                "the number of <pre> elements is unexpected: detect {}",
                pres.len()
            ),
            None,
        )));
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(long_contest_name, problem_id, i + 1);
        let (infile_name, outfile_name) = common::make_next_iofile_name().map_err(|e| {
            Some(Error::new(
                "creating testcase file",
                "failed to generate testcase file's name.",
                Some(Box::new(e.unwrap())),
            ))
        })?;

        ensure_create(&infile_name, &pres[i * 2 + 1].inner_html())?;
        ensure_create(&outfile_name, &pres[i * 2 + 2].inner_html())?;
    }

    print_msg::in_generating_sample_case_finished(long_contest_name, problem_id, pres.len() / 2);

    Ok(())
}

fn download_text(
    long_contest_name: &str,
    problem_id: &str,
    contest_id: &str,
    problem: &str,
) -> reqwest::Result<String> {
    let url = format!(
        "https://beta.atcoder.jp/contests/{}/tasks/{}_{}",
        contest_id, contest_id, problem
    );
    print_msg::in_fetching_problem(long_contest_name, problem_id, &url);
    reqwest::get(&url)?.text()
}
