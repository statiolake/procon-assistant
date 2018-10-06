#[cfg(not(unix))]
use clipboard::{ClipboardContext, ClipboardProvider};

#[cfg(not(unix))]
pub fn set_clipboard(content: String) {
    let mut provider: ClipboardContext =
        ClipboardProvider::new().expect("critical error: cannot get clipboard provider.");
    provider
        .set_contents(content)
        .expect("critical error: cannot set contents to clipboard.");
}

#[cfg(unix)]
pub fn set_clipboard(content: String) {
    use std::io::prelude::*;
    use std::process::{Command, Stdio};
    let mut child = Command::new("xsel")
        .arg("-bi")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("critical error: failed to run xsel");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
    child.wait().expect("critical error: failed to wait xsel");
}
