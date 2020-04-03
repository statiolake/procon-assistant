pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to log in to Aizu Online Judge")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("sorry, log in to Aizu Online Judge is not supported yet")]
    UnsupportedAction,

    #[error("request failed")]
    RequestFailed { source: anyhow::Error },
}

#[allow(dead_code)]
pub fn login(_username: String, _password: String) -> Result<String> {
    Err(Error(ErrorKind::UnsupportedAction))
}

pub fn authenticated_get(url: &str) -> Result<reqwest::blocking::Response> {
    reqwest::blocking::get(url).map_err(|e| Error(ErrorKind::RequestFailed { source: e.into() }))
}
