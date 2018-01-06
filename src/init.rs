use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

pub fn main() -> bool {
    let p: &Path = "main.cpp".as_ref();
    if p.exists() {
        print_error!("file main.cpp already exists.");
        return false;
    }

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

    print_created!("main.cpp");

    return true;
}
