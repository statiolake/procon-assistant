define_error!();
define_error_kind! {
    [UnsupportedAction; (action: String); format!("sorry, {} is not supported for now.", action)];
    [RequestFailed; (); format!("request failed.")];
}

#[allow(dead_code)]
pub fn login(_username: String, _password: String) -> Result<String> {
    return Err(Error::new(ErrorKind::UnsupportedAction(
        "login".to_string(),
    )));
}

pub fn authenticated_get(url: &str) -> Result<reqwest::Response> {
    reqwest::get(url).chain(ErrorKind::RequestFailed())
}
