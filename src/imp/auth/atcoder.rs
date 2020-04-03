use crate::eprintln_debug;
use anyhow::anyhow;
use itertools::Itertools;
use maplit::hashmap;
use reqwest::blocking::{Client, ClientBuilder, RequestBuilder, Response};
use reqwest::header;
use reqwest::header::GetAll;
use reqwest::header::HeaderValue;
use reqwest::redirect::Policy;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use std::collections::HashMap;

const SERVICE_NAME: &str = "atcoder";

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("authenticated operation failed")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to initialize a client")]
    ClientInitializationFailed { source: anyhow::Error },

    #[error("failed to load a session")]
    LoadingSessionFailed { source: anyhow::Error },

    #[error("failed to fetch login page")]
    FetchingLoginPageFailed { source: anyhow::Error },

    #[error("failed to post account info")]
    PostingAccountInfoFailed { source: anyhow::Error },

    #[error("failed to authenticate: invalid username or password")]
    InvalidAuthInfo,

    #[error("invalid HTTP status")]
    HTTPStatusNotOk { status: StatusCode },

    #[error("failed to fetch a page")]
    RequestingError { source: anyhow::Error },
}

struct CookieStore {
    cookies: HashMap<String, String>,
}

impl CookieStore {
    pub fn new() -> CookieStore {
        CookieStore {
            cookies: HashMap::new(),
        }
    }

    pub fn load_from_session() -> anyhow::Result<CookieStore> {
        let cookies = super::load_session_info(SERVICE_NAME)?;
        let cookies = String::from_utf8_lossy(&cookies).into_owned();
        let cookies = cookies
            .lines()
            .map(|l| l.split('\t').map(ToString::to_string).collect_vec())
            .map(|e| (e[0].clone(), e[1].clone()))
            .collect();

        Ok(CookieStore { cookies })
    }

    pub fn save_to_session(&self) -> anyhow::Result<()> {
        let cookies = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}\t{}", k, v))
            .join("\n");
        super::store_session_info(SERVICE_NAME, cookies.as_bytes())?;

        Ok(())
    }

    pub fn with_cookie(&mut self, req: RequestBuilder) -> anyhow::Result<Response> {
        let cookies = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .join(";");
        let cookies = HeaderValue::from_str(&cookies)?;
        let req = req.header(header::COOKIE, cookies);
        let resp = req.send()?;
        self.update_cookie(resp.headers().get_all(header::SET_COOKIE));

        Ok(resp)
    }

    fn update_cookie(&mut self, cookie: GetAll<HeaderValue>) {
        let new_cookies: HashMap<String, String> = cookie
            .into_iter()
            .filter_map(|x| x.to_str().ok())
            .flat_map(|cookiestr| cookiestr.split(';'))
            .filter_map(|raw_value| {
                let mut split = raw_value.splitn(2, '=');
                let name = split.next()?.trim().to_string();
                let value = split.next()?.trim().to_string();
                eprintln_debug!("set-cookie: {} -> '{}'='{}'", raw_value, name, value);
                Some((name, value))
            })
            .collect();
        self.cookies.extend(new_cookies);
    }
}

impl Drop for CookieStore {
    fn drop(&mut self) {
        // we need to ignore the error
        let _ = self.save_to_session();
    }
}

pub fn login(username: &str, password: &str) -> Result<()> {
    let mut store = CookieStore::new();

    // access the login page and get csrf_token
    let csrf_token = access_login_page(&mut store)
        .map_err(|source| Error(ErrorKind::FetchingLoginPageFailed { source }))?;

    // post user authentication info
    let success = post_account_info(&mut store, username, password, &csrf_token)
        .map_err(|source| Error(ErrorKind::PostingAccountInfoFailed { source }))?;

    if !success {
        return Err(Error(ErrorKind::InvalidAuthInfo));
    }

    Ok(())
}

fn new_client() -> Result<Client> {
    ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .map_err(|e| Error(ErrorKind::ClientInitializationFailed { source: e.into() }))
}

fn access_login_page(store: &mut CookieStore) -> anyhow::Result<String> {
    eprintln_debug!("fetching login page");
    let req = new_client()?.get("https://atcoder.jp/login");
    let res = store.with_cookie(req)?;
    result_check(&res)?;
    let csrf_token = parse_csrf_token(res)?;

    Ok(csrf_token)
}

/// Posts account information. Returns true if the login was successful
fn post_account_info(
    store: &mut CookieStore,
    username: &str,
    password: &str,
    csrf_token: &str,
) -> anyhow::Result<bool> {
    let req = new_client()?
        .post("https://atcoder.jp/login")
        .form(&hashmap! {
            "username" => username,
            "password" => password,
            "csrf_token" => csrf_token,
        });
    let res = store.with_cookie(req)?;
    result_check(&res)?;

    Ok(store.cookies["REVEL_FLASH"].contains("success"))
}

fn parse_csrf_token(res: Response) -> anyhow::Result<String> {
    let text = res.text()?;
    let doc = Html::parse_document(&text);
    let sel_csrf_token = Selector::parse("input[name=csrf_token]").unwrap();
    let csrf_token_tag = doc
        .select(&sel_csrf_token)
        .next()
        .ok_or_else(|| anyhow!("failed to find csrf_token"))?;
    let csrf_token = csrf_token_tag
        .value()
        .attr("value")
        .ok_or_else(|| anyhow!("failed to get csrf_token value"))?;

    Ok(csrf_token.to_string())
}

fn result_check(res: &Response) -> Result<()> {
    eprintln_debug!("response: {:?}", res);
    let status = res.status();
    match status {
        StatusCode::OK | StatusCode::FOUND | StatusCode::MOVED_PERMANENTLY => Ok(()),
        _ => Err(Error(ErrorKind::HTTPStatusNotOk { status })),
    }
}

pub fn authenticated_get(url: &str) -> Result<Response> {
    let mut store = CookieStore::load_from_session()
        .map_err(|source| Error(ErrorKind::LoadingSessionFailed { source }))?;

    let req = new_client()?.get(url);
    let res = store
        .with_cookie(req)
        .map_err(|source| Error(ErrorKind::RequestingError { source }))?;

    if cfg!(debug_assertions) {
        let req = new_client().unwrap().get(url);
        let res = store.with_cookie(req).unwrap();
        eprintln_debug!("response: {:?}", res.text().unwrap());
    }

    Ok(res)
}
