use std::fs;
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
    writeln!(f, "int main() {{")?;
    writeln!(f, "")?;
    writeln!(f, "    return 0;")?;
    writeln!(f, "}}")?;
    Ok(())
}

fn generate_clang_complete(p: &Path) -> io::Result<()> {
    let libdir_escaped = common::get_procon_lib_dir()
        .unwrap()
        .display()
        .to_string()
        .escape_default();
    let mut f = File::create(p)?;
    writeln!(f, "-I{}", libdir_escaped)?;
    writeln!(f, "-Wno-old-style-cast")?;
    Ok(())
}

fn generate_vscode_c_cpp_properties(p: &Path) -> io::Result<()> {
    let libdir_escaped = common::get_procon_lib_dir()
        .unwrap()
        .display()
        .to_string()
        .escape_default();
    let mut f = File::create(p)?;
    writeln!(f, r#"{{"#)?;
    writeln!(f, r#"    "configurations": ["#)?;
    writeln!(f, r#"        {{"#)?;
    writeln!(f, r#"            "name": "Win32","#)?;
    writeln!(f, r#"            "browse": {{"#)?;
    writeln!(f, r#"                "path": ["#)?;
    writeln!(f, r#"                    "${{workspaceFolder}}""#)?;
    writeln!(f, r#"                ],"#)?;
    writeln!(
        f,
        r#"                "limitSymbolsToIncludedHeaders": true"#
    )?;
    writeln!(f, r#"            }},"#)?;
    writeln!(f, r#"            "includePath": ["#)?;
    writeln!(f, r#"                "${{workspaceFolder}}","#)?;
    writeln!(f, r#"                "{}""#, libdir_escaped)?;
    writeln!(f, r#"            ],"#)?;
    writeln!(f, r#"            "defines": ["#)?;
    writeln!(f, r#"                "PA_DEBUG","#)?;
    writeln!(f, r#"                "_DEBUG","#)?;
    writeln!(f, r#"                "UNICODE","#)?;
    writeln!(f, r#"                "_UNICODE""#)?;
    writeln!(f, r#"            ],"#)?;
    writeln!(f, r#"            "cStandard": "c11","#)?;
    writeln!(f, r#"            "cppStandard": "c++17","#)?;
    writeln!(f, r#"            "intelliSenseMode": "msvc-x64""#)?;
    writeln!(f, r#"        }}"#)?;
    writeln!(f, r#"    ],"#)?;
    writeln!(f, r#"    "version": 4"#)?;
    writeln!(f, r#"}}"#)?;

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

    fs::create_dir_all(".vscode")
        .map_err(|e| Error::with_cause("generating .vscode", "failed to create directory", box e))?;
    let p = ensure_not_exists(".vscode/c_cpp_properties.json")?;
    generate_vscode_c_cpp_properties(p).map_err(|e| {
        Error::with_cause(
            "generating .vscode/c_cpp_properties.json",
            "failed to write",
            box e,
        )
    })?;
    print_generated!(".vscode/c_cpp_properties.json");

    common::open("main.cpp")?;

    Ok(())
}
