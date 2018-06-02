use std::fs::File;
use std::io::{Read, Write};

use reqwest;
use scraper::{Html, Selector};

use {Error, Result};

pub const ACCESSCODE_FILE: &str = ".accesscode";

pub fn store_revel_session_from_response(
    res: &mut reqwest::Response,
    must_create: bool,
) -> Result<()> {
    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    let cookie = extract_setcookie(setcookie);
    if let Ok(code) = find_revel_session(cookie) {
        store_revel_session(&code, must_create)?;
    }
    Ok(())
}

pub fn store_revel_session(code: &str, must_create: bool) -> Result<()> {
    match File::create(ACCESSCODE_FILE) {
        Ok(mut f) => writeln!(f, "{}", code).unwrap(),
        Err(e) => {
            if must_create {
                return Err(Error::with_cause(
                    "logging in to AtCoder",
                    format!("failed to open {}", ACCESSCODE_FILE),
                    box e,
                ));
            }
        }
    }
    Ok(())
}

pub fn try_login(username: String, password: String) -> Result<String> {
    let (cookie, csrf_token) = get_cookie_and_csrf_token()?;
    let cookie = login_get_cookie(cookie, username, password, csrf_token)?;

    find_revel_session(cookie)
}

pub fn get_with_auth(url: &str) -> reqwest::Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let mut builder = client.get(url);
    if let Ok(mut f) = File::open(ACCESSCODE_FILE)
        .or_else(|_| File::open(format!("../{}", ACCESSCODE_FILE)))
        .or_else(|_| File::open(format!("../../{}", ACCESSCODE_FILE)))
    {
        print_info!("found accesscode file, try to use it.");
        let mut revel_session = String::new();
        f.read_to_string(&mut revel_session).unwrap();
        let mut cookie = reqwest::header::Cookie::new();
        cookie.append("REVEL_SESSION", revel_session.trim().to_string());
        builder.header(cookie);
    }

    builder.send()
}

fn get_cookie_and_csrf_token() -> Result<(Vec<(String, String)>, String)> {
    print_fetching!("login page");
    let client = reqwest::Client::new();
    let mut res = client
        .get("https://beta.atcoder.jp/login")
        .send()
        .map_err(|e| {
            Error::with_cause("logging in to AtCoder", "failed to fetch login page", box e)
        })?;

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
    let doc = Html::parse_document(&res.text().map_err(|e| {
        Error::with_cause(
            "logging in to AtCoder",
            "failed to parse response Html",
            box e,
        )
    })?);
    let sel_csrf_token = Selector::parse("input[name=csrf_token]").unwrap();
    let csrf_token_tag = doc.select(&sel_csrf_token).next().ok_or(Error::new(
        "logging in to AtCoder",
        "failed to get csrf_token tag",
    ))?;

    let csrf_token_tag_value = csrf_token_tag.value().attr("value").ok_or(Error::new(
        "logging in to AtCoder",
        "csrf_token tag has no value attribute!",
    ))?;

    Ok(csrf_token_tag_value.to_string())
}

fn login_get_cookie(
    cookie: Vec<(String, String)>,
    username: String,
    password: String,
    csrf_token: String,
) -> Result<Vec<(String, String)>> {
    print_logging_in!("to AtCoder");
    let client = make_client().map_err(|e| {
        Error::with_cause("logging in to AtCoder", "failed to create client.", box e)
    })?;
    let params: [(&str, &str); 3] = [
        ("username", &username),
        ("password", &password),
        ("csrf_token", &csrf_token),
    ];
    let post_cookie = make_post_cookie(cookie);
    let res = post(client, &params, post_cookie).map_err(|e| {
        Error::with_cause(
            "logging in to AtCoder",
            "failed to post username and password",
            box e,
        )
    })?;

    result_check(&res)?;

    if !is_login_succeeded(&res) {
        return Err(Error::new(
            "logging in to AtCoder",
            "login failed. check your username and password.",
        ));
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
        .ok_or(Error::new(
            "logging in to AtCoder",
            "failed to find REVEL_SESSION.",
        ))
        .map(|c| c.1)
}

fn result_check(res: &reqwest::Response) -> Result<()> {
    match res.status() {
        reqwest::StatusCode::Ok | reqwest::StatusCode::Found => Ok(()),
        _ => Err(Error::new(
            "logging in to AtCoder",
            format!("HTTP status not OK: {:?}", res.status()),
        )),
    }
}
