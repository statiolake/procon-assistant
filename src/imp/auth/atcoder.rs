use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{RequestBuilder, StatusCode};
use scraper::{Html, Selector};

const SERVICE_NAME: &str = "atcoder";

define_error!();
define_error_kind! {
    [InvalidUtf8SessionInfo; (); format!("session info has invalid utf-8.")];
    [RequestingError; (); format!("failed to send your request.")];
    [FetchingLoginPageFailed; (); format!("failed to fetch login page.")];
    [ParsingHtmlFailed; (); format!("failed to parse received HTML.")];
    [GettingCsrfTokenFailed; (); format!("getting csrf token failed.")];
    [CsrfTokenMissingValue; (); format!("csrf token has no attribute value!")];
    [PostingAccountInfoFailed; (); format!("failed to post username and password")];
    [LoginUnsuccessful; (); format!("logging in failed; check your username and password.")];
    [InvalidHeaderValue; (); format!("invalid header value.")];
    [MissingRevelSession; (); format!("failed to find REVEL_SESSION")];
    [StoringRevelSessionFailed; (); format!("failed to store REVEL_SESSION.")];
    [HTTPStatusNotOk; (status: StatusCode); format!("HTTP status not OK: {:?}", status)];
}

pub fn login(username: String, password: String) -> Result<()> {
    let (cookie, csrf_token) = get_cookie_and_csrf_token()?;
    let cookie = login_get_cookie(cookie, username, password, csrf_token)?;
    let revel_session = find_revel_session(cookie)?;
    super::store_session_info(SERVICE_NAME, revel_session.as_bytes())
        .chain(ErrorKind::StoringRevelSessionFailed())?;

    Ok(())
}

pub fn authenticated_get(url: &str) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let mut builder = client.get(url);
    builder = add_auth_info_to_builder_if_possible(builder)?;
    let mut res = builder.send().chain(ErrorKind::RequestingError())?;
    store_revel_session_from_response(&mut res)?;
    Ok(res)
}

fn add_auth_info_to_builder_if_possible(mut builder: RequestBuilder) -> Result<RequestBuilder> {
    fn handle_invalid_utf_8(e: Error) -> Error {
        super::clear_session_info(SERVICE_NAME)
            .expect("critical error: failed to clean session info.");
        print_info!("cleared session info to avoid continuous error.");
        e
    }

    if let Ok(revel_session) = super::load_session_info(SERVICE_NAME) {
        print_info!("found sesion info, try to use it.");
        let revel_session = String::from_utf8(revel_session)
            .chain(ErrorKind::InvalidUtf8SessionInfo())
            .map_err(handle_invalid_utf_8)?
            .trim()
            .to_string();

        builder = builder.header(header::COOKIE, format!("REVEL_SESSION={}", revel_session));
    }

    Ok(builder)
}

fn store_revel_session_from_response(res: &mut reqwest::Response) -> Result<()> {
    let cookie = extract_setcookie(res.headers());
    let revel_session = find_revel_session(cookie)?;
    super::store_session_info(SERVICE_NAME, revel_session.as_bytes())
        .chain(ErrorKind::StoringRevelSessionFailed())
}

fn get_cookie_and_csrf_token() -> Result<(Vec<(String, String)>, String)> {
    print_info!("fetching login page");
    let client = reqwest::Client::new();
    let mut res = client
        .get("https://beta.atcoder.jp/login")
        .send()
        .chain(ErrorKind::FetchingLoginPageFailed())?;

    result_check(&res)?;

    let cookie = get_cookie_from_response(&mut res);

    let csrf_token = get_csrf_token_from_response(&mut res)?;

    Ok((cookie, csrf_token))
}

fn get_cookie_from_response(res: &mut reqwest::Response) -> Vec<(String, String)> {
    extract_setcookie(res.headers())
}

fn get_csrf_token_from_response(res: &mut reqwest::Response) -> Result<String> {
    let doc = res
        .text()
        .map(|res| Html::parse_document(&res))
        .chain(ErrorKind::ParsingHtmlFailed())?;
    let sel_csrf_token = Selector::parse("input[name=csrf_token]").unwrap();
    let csrf_token_tag = doc
        .select(&sel_csrf_token)
        .next()
        .ok_or(Error::new(ErrorKind::GettingCsrfTokenFailed()))?;
    let csrf_token_tag_value = csrf_token_tag
        .value()
        .attr("value")
        .ok_or(Error::new(ErrorKind::CsrfTokenMissingValue()))?;

    Ok(csrf_token_tag_value.to_string())
}

fn login_get_cookie(
    cookie: Vec<(String, String)>,
    username: String,
    password: String,
    csrf_token: String,
) -> Result<Vec<(String, String)>> {
    let client = make_client().expect("critical error: creating client failed.");
    let params: [(&str, &str); 3] = [
        ("username", &username),
        ("password", &password),
        ("csrf_token", &csrf_token),
    ];
    let post_cookie = make_post_cookie(cookie)?;
    print_debug!(true, "post: cookie: {:?}", post_cookie);
    let res = post(client, &params, post_cookie).chain(ErrorKind::PostingAccountInfoFailed())?;

    result_check(&res)?;

    if !is_login_succeeded(&res)? {
        return Err(Error::new(ErrorKind::LoginUnsuccessful()));
    }

    let cookie = extract_setcookie(res.headers());

    Ok(cookie)
}

fn make_client() -> reqwest::Result<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::RedirectPolicy::none())
        .build()
}

fn make_post_cookie(cookie: Vec<(String, String)>) -> Result<HeaderValue> {
    let mut post_cookie = Vec::new();
    for (head, value) in cookie {
        post_cookie.push(format!("{}={}", head, value));
    }
    HeaderValue::from_str(&post_cookie.join("; ")).chain(ErrorKind::InvalidHeaderValue())
}

fn post(
    client: reqwest::Client,
    params: &[(&str, &str)],
    post_cookie: HeaderValue,
) -> reqwest::Result<reqwest::Response> {
    client
        .post("https://beta.atcoder.jp/login")
        .form(params)
        .header(header::COOKIE, post_cookie)
        .send()
}

fn is_login_succeeded(res: &reqwest::Response) -> Result<bool> {
    let loc = res.headers().get(header::LOCATION).unwrap();
    loc.to_str()
        .map(|loc| loc == "/")
        .chain(ErrorKind::InvalidHeaderValue())
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
        .ok_or(Error::new(ErrorKind::MissingRevelSession()))
        .map(|c| c.1)
}

fn result_check(res: &reqwest::Response) -> Result<()> {
    print_debug!(true, "response: {:?}", res);
    match res.status() {
        StatusCode::OK | StatusCode::FOUND => Ok(()),
        _ => Err(Error::new(ErrorKind::HTTPStatusNotOk(res.status()))),
    }
}
