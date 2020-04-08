use crate::eprintln_debug;
use crate::imp::config;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};

pub mod aoj;
pub mod atcoder;

pub fn ask_account_info(service_name: &str) -> (String, String) {
    print!("  {} Username: ", service_name);
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("  {} Password: ", service_name);
    io::stdout().flush().unwrap();
    let password =
        rpassword::read_password().expect("critical error: failed to read your password input");

    (username.trim().into(), password.trim().into())
}

pub fn place_to_store(service_name: &str) -> PathBuf {
    let mut path = config::config_dir();
    path.push("auth_info");
    fs::create_dir_all(&path).expect("critical error: failed to create auth_info directory");
    path.push(service_name);

    path
}

pub fn clear_session_info(service_name: &str) -> io::Result<()> {
    let place = place_to_store(service_name);
    if place.exists() {
        fs::remove_file(&place)?;
    }
    Ok(())
}

pub fn store_session_info(service_name: &str, contents: &[u8]) -> io::Result<()> {
    clear_session_info(service_name).expect("critical error: failed to clear session info");

    let place = place_to_store(service_name);
    if let Some(parent) = place.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(place)?.write_all(contents)?;

    Ok(())
}

pub fn load_session_info(service_name: &str) -> io::Result<Vec<u8>> {
    let place = place_to_store(service_name);
    let mut contents = Vec::new();
    eprintln_debug!(
        "loading session info for {} from {}",
        service_name,
        place.display()
    );
    File::open(place)?.read_to_end(&mut contents)?;

    Ok(contents)
}
