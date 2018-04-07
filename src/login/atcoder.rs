use reqwest;
use rpassword;
use scraper::{Html, Selector};

use std::fs::File;
use std::io;
use std::io::Write;

use Error;
use Result;

pub fn main() -> Result<()> {
    let (username, password) = ask_account_info()?;
    let code = try_login(username, password)?;
    let mut f = File::create(".accesscode").map_err(|e| {
        Error::with_cause("logging in to AtCoder", "failed to open .accesscode", box e)
    })?;
    writeln!(f, "{}", code).unwrap();
    print_finished!("fetching code; successfully saved.");
    Ok(())
}

pub fn store_cookie(res: &mut reqwest::Response) {
    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    let cookie = extract_setcookie(setcookie);
    if let Ok(code) = find_revel_session(cookie) {
        if let Ok(mut f) = File::create(".accesscode") {
            writeln!(f, "{}", code).unwrap();
        }
    }
}

fn try_login(username: String, password: String) -> Result<String> {
    let (cookie, csrf_token) = get_cookie_and_csrf_token()?;
    let cookie = login_get_cookie(cookie, username, password, csrf_token)?;
    find_revel_session(cookie)
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

    let cookie;
    {
        let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
        cookie = extract_setcookie(setcookie);
    }

    let csrf_token;
    {
        let doc = Html::parse_document(&res.text().map_err(|e| {
            Error::with_cause(
                "logging in to AtCoder",
                "failed to parse response Html",
                box e,
            )
        })?);
        let sel_csrf_token = Selector::parse("input[name=csrf_token]").unwrap();
        csrf_token = doc.select(&sel_csrf_token)
            .next()
            .ok_or(Error::new(
                "logging in to AtCoder",
                "failed to get csrf_token tag",
            ))?
            .value()
            .attr("value")
            .ok_or(Error::new(
                "logging in to AtCoder",
                "csrf_token tag has no value attribute!",
            ))?
            .to_string();
    }

    Ok((cookie, csrf_token))
}

fn login_get_cookie(
    cookie: Vec<(String, String)>,
    username: String,
    password: String,
    csrf_token: String,
) -> Result<Vec<(String, String)>> {
    print_logging_in!("to AtCoder");
    let params = [
        ("username", &username),
        ("password", &password),
        ("csrf_token", &csrf_token),
    ];
    let mut postcookie = reqwest::header::Cookie::new();
    for c in cookie.iter() {
        postcookie.append(c.0.clone(), c.1.clone());
    }

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::RedirectPolicy::none())
        .build()
        .map_err(|e| {
            Error::with_cause("logging in to AtCoder", "failed to create client.", box e)
        })?;

    let res = client
        .post("https://beta.atcoder.jp/login")
        .form(&params)
        .header(postcookie)
        .send()
        .map_err(|e| {
            Error::with_cause(
                "logging in to AtCoder",
                "failed to post username and password",
                box e,
            )
        })?;

    result_check(&res)?;
    let setcookie: &reqwest::header::SetCookie = res.headers().get().unwrap();
    let cookie = extract_setcookie(setcookie);

    Ok(cookie)
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

fn ask_account_info() -> Result<(String, String)> {
    print!("  AtCoder Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("  AtCoder Password: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().map_err(|e| {
        Error::with_cause(
            "fetching login page",
            "failed to read your password input.",
            box e,
        )
    })?;

    Ok((username.trim().into(), password.trim().into()))
}
