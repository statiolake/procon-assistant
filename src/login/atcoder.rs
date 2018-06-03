use imp::auth;
use imp::auth::atcoder;

define_error!();
define_error_kind! {
    [LoginFailed; (); "failed to login."];
    [StoreRevelSessionFailed; (); "failed to store REVEL_SESSION."];
}

pub fn main() -> Result<()> {
    let (username, password) = auth::ask_account_info("AtCoder");
    let code = atcoder::try_login(username, password).chain(ErrorKind::LoginFailed())?;
    atcoder::store_revel_session(&code, true).chain(ErrorKind::StoreRevelSessionFailed())?;
    print_finished!("fetching code; successfully saved.");
    Ok(())
}
