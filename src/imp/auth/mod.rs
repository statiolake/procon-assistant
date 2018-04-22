pub mod atcoder;

use std::io;
use std::io::Write;

use rpassword;

use {Error, Result};
pub fn ask_account_info(service_name: &str) -> Result<(String, String)> {
    print!("  {} Username: ", service_name);
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("  {} Password: ", service_name);
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
