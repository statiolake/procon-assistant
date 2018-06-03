use reqwest;
use scraper::{Html, Selector};

use super::print_msg;
use imp::test_case::TestCaseFile;

define_error!();
define_error_kind!{
    [FetchingProblemFailed; (problem_id: String); format!("failed to fetch the problem `{}'", problem_id)];
    [UnexpectedNumberOfPreTag; (detected: usize); format!("unexpected number of <pre>: {}", detected)];
    [CouldNotDetermineTestCaseFileName; (); format!("failed to determine testcase file name.")];
    [TestCaseCreationFailed; (); format!("failed to create testcase.")];
}

const CONTEST: &str = "Aizu Online Judge";

pub fn main(problem_id: &str) -> Result<()> {
    let text =
        download_text(problem_id).chain(ErrorKind::FetchingProblemFailed(problem_id.to_string()))?;

    let document = Html::parse_document(&text);
    let sel_pre = Selector::parse("pre").unwrap();

    let mut pres: Vec<_> = document.select(&sel_pre).collect();
    if pres.len() <= 1 {
        return Err(Error::new(ErrorKind::UnexpectedNumberOfPreTag(pres.len())));
    }

    if pres.len() % 2 == 1 {
        pres = pres.into_iter().skip(1).collect();
    }

    for i in 0..(pres.len() / 2) {
        print_msg::in_generating_sample_case(CONTEST, problem_id, i + 1);
        let tsf = TestCaseFile::new_with_next_unused_name()
            .chain(ErrorKind::CouldNotDetermineTestCaseFileName())?;
        tsf.create_with_contents(pres[i * 2].inner_html(), pres[i * 2 + 1].inner_html())
            .chain(ErrorKind::TestCaseCreationFailed())?;
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
