use anyhow::bail;
use anyhow::{Context, Result};

#[allow(dead_code)]
pub fn login(_username: String, _password: String) -> Result<String> {
    bail!("logging in to AOJ is currently not supported");
}

pub fn authenticated_get(url: &str) -> Result<reqwest::blocking::Response> {
    reqwest::blocking::get(url).with_context(|| format!("failed to get {} without login", url))
}
