use reqwest;
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
    [MissingRevelSession; (); format!("failed to find REVEL_SESSION")];
    [StoringRevelSessionFailed; (); format!("failed to store REVEL_SESSION.")];
    [HTTPStatusNotOk; (status: reqwest::StatusCode); format!("HTTP status not OK: {:?}", status)];
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
    add_auth_info_to_builder_if_possible(&mut builder)?;
    let mut res = builder.send().chain(ErrorKind::RequestingError())?;
    store_revel_session_from_response(&mut res)?;
    Ok(res)
}

fn add_auth_info_to_builder_if_possible(builder: &mut reqwest::RequestBuilder) -> Result<()> {
    fn handle_invalid_utf_8(e: Error) -> Error {
        super::clear_session_info(SERVICE_NAME)
            .expect("critical error: failed to clean session info.");
        print_info!(true, "cleared session info to avoid continuous error.");
        e
    }

    if let Ok(revel_session) = super::load_session_info(SERVICE_NAME) {
        print_info!(true, "found sesion info, try to use it.");
        let revel_session = String::from_utf8(revel_session)
            .chain(ErrorKind::InvalidUtf8SessionInfo())
            .map_err(handle_invalid_utf_8)?
            .trim()
            .to_string();

        let mut cookie = reqwest::header::Cookie::new();
        cookie.append("REVEL_SESSION", revel_session);
        builder.header(cookie);
    }

    Ok(())
}

fn store_revel_session_from_response(res: &mut reqwest::Response) -> Result<()> {
    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    let cookie = extract_setcookie(setcookie);
    let revel_session = find_revel_session(cookie)?;
    super::store_session_info(SERVICE_NAME, revel_session.as_bytes())
        .chain(ErrorKind::StoringRevelSessionFailed())
}

fn get_cookie_and_csrf_token() -> Result<(Vec<(String, String)>, String)> {
    print_info!(true, "fetching login page");
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
    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    extract_setcookie(setcookie)
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
    let post_cookie = make_post_cookie(cookie);
    let res = post(client, &params, post_cookie).chain(ErrorKind::PostingAccountInfoFailed())?;

    result_check(&res)?;

    if !is_login_succeeded(&res) {
        return Err(Error::new(ErrorKind::LoginUnsuccessful()));
    }

    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    let cookie = extract_setcookie(setcookie);

    Ok(cookie)
}

fn make_client() -> reqwest::Result<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::RedirectPolicy::none())
        .build()
}

fn make_post_cookie(cookie: Vec<(String, String)>) -> reqwest::header::Cookie {
    let mut post_cookie = reqwest::header::Cookie::new();
    for (head, value) in cookie.iter() {
        post_cookie.append(head.clone(), value.clone());
    }
    post_cookie
}

fn post(
    client: reqwest::Client,
    params: &[(&str, &str)],
    post_cookie: reqwest::header::Cookie,
) -> reqwest::Result<reqwest::Response> {
    client
        .post("https://beta.atcoder.jp/login")
        .form(params)
        .header(post_cookie)
        .send()
}

fn is_login_succeeded(res: &reqwest::Response) -> bool {
    let loc: &reqwest::header::Location = res.headers().get().unwrap();
    &**loc == "/"
}

fn extract_setcookie(setcookie: &reqwest::header::SetCookie) -> Vec<(String, String)> {
    setcookie
        .iter()
        .map(|cookiestr| {
            let split: Vec<_> = cookiestr.split('=').collect();
            (
                split[0].into(),
                split[1].chars().take_while(|&ch| ch != ';').collect(),
            )
        })
        .collect()
}

fn find_revel_session(cookie: Vec<(String, String)>) -> Result<String> {
    cookie
        .into_iter()
        .find(|c| c.0 == "REVEL_SESSION")
        .ok_or(Error::new(ErrorKind::MissingRevelSession()))
        .map(|c| c.1)
}

fn result_check(res: &reqwest::Response) -> Result<()> {
    match res.status() {
        reqwest::StatusCode::Ok | reqwest::StatusCode::Found => Ok(()),
        _ => Err(Error::new(ErrorKind::HTTPStatusNotOk(res.status()))),
    }
}
