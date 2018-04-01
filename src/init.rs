use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

fn spawn(name: &str) {
    let successful = Command::new("open").arg(name).spawn().is_ok();

    if !successful {
        print_error!("failed to open main.cpp file. please manually open the file.");
    }
}

pub fn main() -> bool {
    let p: &Path = "main.cpp".as_ref();
    if p.exists() {
        print_error!("file main.cpp already exists.");
        return false;
    }

    {
        let mut f = match File::create("main.cpp") {
            Ok(f) => f,
            Err(_) => return false,
        };

        writeln!(f, "#include <bits/stdc++.h>").unwrap();
        writeln!(f, "using namespace std;").unwrap();
        writeln!(f, "int main() {{").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "    return 0;").unwrap();
        writeln!(f, "}}").unwrap();
    }
    print_created!("main.cpp");

    spawn("main.cpp");

    return true;
}
