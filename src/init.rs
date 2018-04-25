use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use common;

use Error;
use Result;

fn ensure_not_exists(p: &str) -> Result<&Path> {
    let p = Path::new(p);
    if p.exists() {
        Err(Error::new(
            format!("creating {}", p.display()),
            format!("file {} already exists.", p.display()),
        ))
    } else {
        Ok(p)
    }
}

fn generate_main_cpp(p: &Path) -> io::Result<()> {
    let mut f = File::create(p)?;

    writeln!(f, "#include <bits/stdc++.h>")?;
    writeln!(f, "#include \"prelude.hpp\"")?;
    writeln!(f, "using namespace std;")?;
    writeln!(f, "using ll = long long;")?;
    writeln!(f, "int main() {{")?;
    writeln!(f, "")?;
    writeln!(f, "    return 0;")?;
    writeln!(f, "}}")?;
    Ok(())
}

fn generate_clang_complete(p: &Path) -> io::Result<()> {
    let mut f = File::create(p)?;
    writeln!(f, "-I{}", common::get_procon_lib_dir().unwrap().display())?;
    writeln!(f, "-Wno-old-style-cast")?;
    Ok(())
}

pub fn main() -> Result<()> {
    let p = ensure_not_exists(".clang_complete")?;
    generate_clang_complete(p)
        .map_err(|e| Error::with_cause("generating .clang_complete", "failed to write.", box e))?;
    print_generated!(".clang_complete");

    let p = ensure_not_exists("main.cpp")?;
    generate_main_cpp(p)
        .map_err(|e| Error::with_cause("generating main.cpp", "failed to write.", box e))?;
    print_generated!("main.cpp");

    common::open("main.cpp")?;

    Ok(())
}
