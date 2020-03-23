use crate::{print_debug, print_info};
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{RequestBuilder, StatusCode};
use scraper::{Html, Selector};

const SERVICE_NAME: &str = "atcoder";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("authenticated operation failed")]
pub struct Error(ErrorKind);

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("session info has invalid utf-8")]
    InvalidUtf8SessionInfo { source: anyhow::Error },

    #[error("failed to send your request")]
    RequestingError { source: anyhow::Error },

    #[error("failed to fetch login page")]
    FetchingLoginPageFailed { source: anyhow::Error },

    #[error("failed to parse received HTML")]
    ParsingHtmlFailed { source: anyhow::Error },

    #[error("getting csrf token failed")]
    GettingCsrfTokenFailed,

    #[error("csrf token has no attribute value")]
    CsrfTokenMissingValue,

    #[error("failed to post username and password")]
    PostingAccountInfoFailed { source: anyhow::Error },

    #[error("logging in failed; check your username and password")]
    LoginUnsuccessful,

    #[error("invalid header value")]
    InvalidHeaderValue { source: anyhow::Error },

    #[error("failed to find REVEL_SESSION")]
    MissingRevelSession,

    #[error("failed to store REVEL_SESSION")]
    StoringRevelSessionFailed { source: anyhow::Error },

    #[error("HTTP status not OK: {:?}", status)]
    HTTPStatusNotOk { status: StatusCode },
}

pub fn login(quiet: bool, username: String, password: String) -> Result<()> {
    let (cookie, csrf_token) = get_cookie_and_csrf_token(quiet)?;
    let cookie = login_get_cookie(cookie, username, password, csrf_token)?;
    let revel_session = find_revel_session(cookie)?;
    super::store_session_info(SERVICE_NAME, revel_session.as_bytes())
        .map_err(|e| Error(ErrorKind::StoringRevelSessionFailed { source: e.into() }))?;

    Ok(())
}

pub fn authenticated_get(quiet: bool, url: &str) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let mut builder = client.get(url);
    builder = add_auth_info_to_builder_if_possible(quiet, builder)?;
    let mut res = async_std::task::block_on(builder.send())
        .map_err(|e| Error(ErrorKind::RequestingError { source: e.into() }))?;
    store_revel_session_from_response(&mut res)?;
    Ok(res)
}

fn add_auth_info_to_builder_if_possible(
    quiet: bool,
    mut builder: RequestBuilder,
) -> Result<RequestBuilder> {
    fn cleanup(quiet: bool) {
        super::clear_session_info(SERVICE_NAME)
            .expect("critical error: failed to clean session info");
        print_info!(!quiet, "cleared session info to avoid continuous error");
    }

    if let Ok(revel_session) = super::load_session_info(quiet, SERVICE_NAME) {
        print_info!(!quiet, "found sesion info, try to use it");
        let revel_session = match String::from_utf8(revel_session) {
            Ok(v) => v.trim().to_string(),
            Err(e) => {
                cleanup(quiet);
                return Err(Error(ErrorKind::InvalidUtf8SessionInfo {
                    source: e.into(),
                }));
            }
        };

        builder = builder.header(header::COOKIE, format!("REVEL_SESSION={}", revel_session));
    }

    Ok(builder)
}

fn store_revel_session_from_response(res: &mut reqwest::Response) -> Result<()> {
    let cookie = extract_setcookie(res.headers());
    let revel_session = find_revel_session(cookie)?;
    super::store_session_info(SERVICE_NAME, revel_session.as_bytes())
        .map_err(|e| Error(ErrorKind::StoringRevelSessionFailed { source: e.into() }))
}

fn get_cookie_and_csrf_token(quiet: bool) -> Result<(Vec<(String, String)>, String)> {
    print_info!(!quiet, "fetching login page");
    let client = reqwest::Client::new();
    let res = async_std::task::block_on(client.get("https://beta.atcoder.jp/login").send())
        .map_err(|e| Error(ErrorKind::FetchingLoginPageFailed { source: e.into() }))?;

    result_check(&res)?;

    let cookie = extract_setcookie(res.headers());
    let csrf_token = get_csrf_token_from_response(res)?;

    Ok((cookie, csrf_token))
}

fn get_csrf_token_from_response(res: reqwest::Response) -> Result<String> {
    let doc = async_std::task::block_on(res.text())
        .map(|res| Html::parse_document(&res))
        .map_err(|e| Error(ErrorKind::ParsingHtmlFailed { source: e.into() }))?;
    let sel_csrf_token = Selector::parse("input[name=csrf_token]").unwrap();
    let csrf_token_tag = doc
        .select(&sel_csrf_token)
        .next()
        .ok_or_else(|| Error(ErrorKind::GettingCsrfTokenFailed))?;
    let csrf_token_tag_value = csrf_token_tag
        .value()
        .attr("value")
        .ok_or_else(|| Error(ErrorKind::CsrfTokenMissingValue))?;

    Ok(csrf_token_tag_value.to_string())
}

fn login_get_cookie(
    cookie: Vec<(String, String)>,
    username: String,
    password: String,
    csrf_token: String,
) -> Result<Vec<(String, String)>> {
    let client = make_client().expect("critical error: creating client failed");
    let params: [(&str, &str); 3] = [
        ("username", &username),
        ("password", &password),
        ("csrf_token", &csrf_token),
    ];
    let post_cookie = make_post_cookie(cookie)?;
    print_debug!(true, "post: cookie: {:?}", post_cookie);
    let res = post(client, &params, post_cookie)
        .map_err(|e| Error(ErrorKind::PostingAccountInfoFailed { source: e.into() }))?;

    result_check(&res)?;

    if !is_login_succeeded(&res)? {
        return Err(Error(ErrorKind::LoginUnsuccessful));
    }

    let cookie = extract_setcookie(res.headers());

    Ok(cookie)
}

fn make_client() -> reqwest::Result<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

fn make_post_cookie(cookie: Vec<(String, String)>) -> Result<HeaderValue> {
    let mut post_cookie = Vec::new();
    for (head, value) in cookie {
        post_cookie.push(format!("{}={}", head, value));
    }
    HeaderValue::from_str(&post_cookie.join("; "))
        .map_err(|e| Error(ErrorKind::InvalidHeaderValue { source: e.into() }))
}

fn post(
    client: reqwest::Client,
    params: &[(&str, &str)],
    post_cookie: HeaderValue,
) -> reqwest::Result<reqwest::Response> {
    let builder = client
        .post("https://beta.atcoder.jp/login")
        .form(params)
        .header(header::COOKIE, post_cookie);
    async_std::task::block_on(builder.send())
}

fn is_login_succeeded(res: &reqwest::Response) -> Result<bool> {
    let loc = res.headers().get(header::LOCATION).unwrap();
    loc.to_str()
        .map(|loc| loc == "/")
        .map_err(|e| Error(ErrorKind::InvalidHeaderValue { source: e.into() }))
}

fn extract_setcookie(header: &HeaderMap) -> Vec<(String, String)> {
    let mut res: Vec<_> = header
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|x| x.to_str().ok())
        .flat_map(|cookiestr| cookiestr.split(';'))
        .filter_map(|raw_value| {
            let mut split = raw_value.splitn(2, '=');
            let name = split.next()?.trim().to_string();
            let value = split.next()?.trim().to_string();
            print_debug!(true, "set-cookie: {} -> '{}'='{}'", raw_value, name, value);
            Some((name, value))
        })
        .collect();
    res.sort();
    res.dedup();
    res
}

fn find_revel_session(cookie: Vec<(String, String)>) -> Result<String> {
    cookie
        .into_iter()
        .find(|c| c.0 == "REVEL_SESSION")
        .ok_or_else(|| Error(ErrorKind::MissingRevelSession))
        .map(|c| c.1)
}

fn result_check(res: &reqwest::Response) -> Result<()> {
    print_debug!(true, "response: {:?}", res);
    let status = res.status();
    match status {
        StatusCode::OK | StatusCode::FOUND => Ok(()),
        _ => Err(Error(ErrorKind::HTTPStatusNotOk { status })),
    }
}
