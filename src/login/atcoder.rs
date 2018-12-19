use crate::imp::auth;
use crate::imp::auth::atcoder as auth_atcoder;

define_error!();
define_error_kind! {
    [LoginFailed; (); "failed to login."];
}

pub fn main(quiet: bool) -> Result<()> {
    let (username, password) = auth::ask_account_info("AtCoder");
    print_logging_in!("to AtCoder");
    auth_atcoder::login(quiet, username, password).chain(ErrorKind::LoginFailed())?;
    print_finished!("fetching code; successfully saved.");
    Ok(())
}
