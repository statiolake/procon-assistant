pub fn err_in_fetching_tasks(_contest: &str, error: &str) {
    print_error!("failed to fetch tasks.");
    print_info!("due to {}", error);
}

pub fn in_fetching_tasks(contest: &str) {
    print_fetching!("{}", contest);
}
