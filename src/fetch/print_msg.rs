pub fn err_in_fetching_problem(_contest: &str, _id: &str, error: &str) {
    print_error!("failed to fetch a problem.");
    print_info!("due to {}", error);
}

pub fn err_in_parsing_problem(_contest: &str, _id: &str, error: &str) {
    print_error!("failed to parse a problem.");
    print_info!("due to {:?}", error);
}

pub fn in_fetching_problem(contest: &str, id: &str, url: &str) {
    print_fetching!("{} id {} (at {})", contest, id, url);
}

pub fn in_generating_sample_case(_contest: &str, _id: &str, case_number: usize) {
    print_generating!("Sample Case {}", case_number);
}

pub fn in_generating_sample_case_finished(contest: &str, id: &str, total_case_number: usize) {
    print_finished!(
        "Generating {} Test Case(s) at {} id {}",
        total_case_number,
        contest,
        id
    );
}
