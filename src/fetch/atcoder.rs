use reqwest;
use scraper::{Html, Selector};

use super::print_msg;
use imp::auth::atcoder;
use imp::test_case::TestCaseFile;

use {Error, Result};

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

pub fn main(problem_id: &str) -> Result<()> {
    let long_contest_name;
    let problem;
    let text = if problem_id.len() != 7 {
        long_contest_name = "unknown";
        problem = "unknown";
        download_text_by_url("Unknown", "Unknown", problem_id)
    } else {
        let contest_name = &problem_id[0..3];
        let contest_id = &problem_id[0..6];
        long_contest_name = get_long_contest_name(contest_name)?;
        problem = &problem_id[6..7];
        download_text(long_contest_name, problem_id, contest_id, problem)
    }.map_err(|e| {
        Error::with_cause(
            "downloading html",
            format!(
                "failed to fetch the problem {} problem {}",
                long_contest_name, problem
            ),
            box e,
        )
    })?;

    let document = Html::parse_document(&text);
    let sel_div_task_statement = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let div_task_statement_not_found =
        Error::new("parsing problem html", "failed to get div#task-statement");

    let span_lang_ja_not_found =
        Error::new("parsing problem html", "failed to get div#span.lang-ja");

    let pres: Vec<_> = document
        .select(&sel_div_task_statement)
        .next()
        .ok_or(div_task_statement_not_found)?
        .select(&sel_span_ja)
        .next()
        .ok_or(span_lang_ja_not_found)?
        .select(&sel_pre)
        .collect();

    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Error::new(
            "parsing problem html",
            format!(
                "the number of <pre> elements is unexpected: detect {}",
                pres.len()
            ),
        ));
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(long_contest_name, problem_id, i + 1);
        let tsf = TestCaseFile::new_with_next_unused_name()?;
        tsf.create_with_contents(pres[i * 2 + 1].inner_html(), pres[i * 2 + 2].inner_html())?;
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

    download_text_by_url(long_contest_name, problem_id, &url)
}

fn download_text_by_url(
    long_contest_name: &str,
    problem_id: &str,
    url: &str,
) -> reqwest::Result<String> {
    print_msg::in_fetching_problem(long_contest_name, problem_id, &url);
    let mut res = atcoder::get_with_auth(url)?;
    atcoder::store_revel_session_from_response(&mut res, false).ok();
    res.text()
}
