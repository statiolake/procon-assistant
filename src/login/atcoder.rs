use imp::auth;
use imp::auth::atcoder;

use Result;

pub fn main() -> Result<()> {
    let (username, password) = auth::ask_account_info("AtCoder")?;
    let code = atcoder::try_login(username, password)?;
    atcoder::store_revel_session(&code, true)?;
    print_finished!("fetching code; successfully saved.");
    Ok(())
}
