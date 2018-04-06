use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use common::open;

use Error;
use Result;

fn ensure_not_exists(p: &str) -> Result<&Path> {
    let p = Path::new(p);
    if p.exists() {
        Err(Error::new(
            "creating main.cpp",
            "file main.cpp already exists.",
        ))
    } else {
        Ok(p)
    }
}

fn generate_main_cpp(p: &Path) -> io::Result<()> {
    let mut f = File::create(p)?;

    writeln!(f, "#include <bits/stdc++.h>")?;
    writeln!(f, "using namespace std;")?;
    writeln!(f, "int main() {{")?;
    writeln!(f, "")?;
    writeln!(f, "    return 0;")?;
    writeln!(f, "}}")?;
    Ok(())
}

pub fn main() -> Result<()> {
    let p = ensure_not_exists("main.cpp")?;
    generate_main_cpp(p)
        .map_err(|e| Error::with_cause("generating main.cpp", "failed to write.", box e))?;
    print_generated!("main.cpp");

    open("main.cpp")?;

    Ok(())
}
