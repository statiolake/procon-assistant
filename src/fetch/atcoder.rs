use reqwest;
use scraper::{Html, Selector};

use super::print_msg;
use imp::auth::atcoder;
use imp::test_case::TestCaseFile;

define_error!();
define_error_kind! {
    [UnknownContestName; (contest_name: String); format!("unknown contest-name: `{}'", contest_name)];
    [FetchingProblemFailed; (long_contest_name: String, problem: String); format!(
        "failed to fetch the problem: {} problem {}", long_contest_name, problem
    )];
    [FindingTagFailed; (selector: String); format!("missing tag: failed to find `{}'\nmaybe failed to login?", selector)];
    [UnexpectedNumberOfPreTag; (detected: usize); format!("unexpected number of <pre>: {}", detected)];
    [CouldNotDetermineTestCaseFileName; (); format!("failed to determine testcase file name.")];
    [TestCaseCreationFailed; (); format!("failed to create testcase.")];
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
    }.chain(ErrorKind::FetchingProblemFailed(
        long_contest_name.into(),
        problem.into(),
    ))?;

    let document = Html::parse_document(&text);
    let sel_div_task_statement = Selector::parse("div#task-statement").unwrap();
    let sel_span_ja = Selector::parse("span.lang-ja").unwrap();
    let sel_pre = Selector::parse("pre").unwrap();

    let pres: Vec<_> = document
        .select(&sel_div_task_statement)
        .next()
        .ok_or(Error::new(ErrorKind::FindingTagFailed(
            "div#task-statement".into(),
        )))?
        .select(&sel_span_ja)
        .next()
        .ok_or(Error::new(ErrorKind::FindingTagFailed(
            "div#span.lang-ja".into(),
        )))?
        .select(&sel_pre)
        .collect();

    if pres.len() <= 1 || (pres.len() - 1) % 2 != 0 {
        return Err(Error::new(ErrorKind::UnexpectedNumberOfPreTag(pres.len())));
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(long_contest_name, problem_id, i + 1);
        let tsf = TestCaseFile::new_with_next_unused_name()
            .chain(ErrorKind::CouldNotDetermineTestCaseFileName())?;
        tsf.create_with_contents(pres[i * 2 + 1].inner_html(), pres[i * 2 + 2].inner_html())
            .chain(ErrorKind::TestCaseCreationFailed())?;
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
