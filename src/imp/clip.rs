#[cfg(not(unix))]
pub fn set_clipboard(content: String) {
    use clipboard::{ClipboardContext, ClipboardProvider};
    let mut provider: ClipboardContext =
        ClipboardProvider::new().expect("critical error: cannot get clipboard provider");
    provider
        .set_contents(content)
        .expect("critical error: cannot set contents to clipboard");
}

#[cfg(unix)]
pub fn set_clipboard(content: String) {
    fn run_xclip(content: &[u8], clipboard_type: &str) {
        use std::io::prelude::*;
        use std::process::{Command, Stdio};
        let mut child = Command::new("xclip")
            .arg("-i")
            .arg("-sel")
            .arg(clipboard_type)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("critical error: failed to run xclip");
        child.stdin.take().unwrap().write_all(content).unwrap();
        child.wait().expect("critical error: failed to wait xsel");
    }

    // primary selection --- paste with middle click
    run_xclip(content.as_bytes(), "p");

    // clipboard selection --- paste with Ctrl + V
    run_xclip(content.as_bytes(), "c");
}
