pub mod atcoder;

use std::io;
use std::io::Write;

use rpassword;

pub fn ask_account_info(service_name: &str) -> (String, String) {
    print!("  {} Username: ", service_name);
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("  {} Password: ", service_name);
    io::stdout().flush().unwrap();
    let password =
        rpassword::read_password().expect("critical error: failed to read your password input.");

    (username.trim().into(), password.trim().into())
}
