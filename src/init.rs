use config::ConfigFile;
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
    writeln!(f, "#include \"prelude.hpp\"")?;
    writeln!(f, "")?;
    writeln!(f, "#include <bits/stdc++.h>")?;
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
    writeln!(f, r#"        }},"#)?;
    writeln!(f, r#"        {{"#)?;
    writeln!(f, r#"            "name": "Linux","#)?;
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
    writeln!(f, r#"            "intelliSenseMode": "clang-x64","#)?;
    writeln!(f, r#"            "compilerPath": "/usr/bin/clang""#)?;
    writeln!(f, r#"        }}"#)?;
    writeln!(f, r#"    ],"#)?;
    writeln!(f, r#"    "version": 4"#)?;
    writeln!(f, r#"}}"#)?;

    Ok(())
}

fn generate_vscode_tasks_json(p: &Path) -> io::Result<()> {
    let mut f = File::create(p)?;

    writeln!(f, r#"{{"#)?;
    writeln!(
        f,
        r#"    // See https://go.microsoft.com/fwlink/?LinkId=733558"#
    )?;
    writeln!(
        f,
        r#"    // for the documentation about the tasks.json format"#
    )?;
    writeln!(f, r#"    "version": "2.0.0","#)?;
    writeln!(f, r#"    "tasks": ["#)?;
    writeln!(f, r#"        {{"#)?;
    writeln!(f, r#"            "label": "procon-assistant compile","#)?;
    writeln!(f, r#"            "type": "shell","#)?;
    writeln!(f, r#"            "command": "procon-assistant","#)?;
    writeln!(f, r#"            "args": ["#)?;
    writeln!(f, r#"                "compile""#)?;
    writeln!(f, r#"            ]"#)?;
    writeln!(f, r#"        }}"#)?;
    writeln!(f, r#"    ]"#)?;
    writeln!(f, r#"}}"#)?;
    Ok(())
}

fn generate_vscode_launch_json(p: &Path) -> io::Result<()> {
    let mut f = File::create(p)?;

    writeln!(f, r#"{{"#)?;
    writeln!(
        f,
        r#"    // IntelliSense を使用して利用可能な属性を学べます。"#
    )?;
    writeln!(
        f,
        r#"    // 既存の属性の説明をホバーして表示します。"#
    )?;
    writeln!(f, r#"    // 詳細情報は次を確認してください: https://go.microsoft.com/fwlink/?linkid=830387"#)?;
    writeln!(f, r#"    "version": "0.2.0","#)?;
    writeln!(f, r#"    "configurations": ["#)?;
    writeln!(f, r#"        {{"#)?;
    writeln!(f, r#"            "name": "(Windows) Launch","#)?;
    writeln!(f, r#"            "type": "cppvsdbg","#)?;
    writeln!(f, r#"            "request": "launch","#)?;
    writeln!(
        f,
        r#"            "program": "${{workspaceFolder}}/main.exe","#
    )?;
    writeln!(f, r#"            "args": [],"#)?;
    writeln!(f, r#"            "stopAtEntry": false,"#)?;
    writeln!(f, r#"            "cwd": "${{workspaceFolder}}","#)?;
    writeln!(
        f,
        r#"            "preLaunchTask": "procon-assistant compile","#
    )?;
    writeln!(f, r#"            "environment": [],"#)?;
    writeln!(f, r#"            "externalConsole": true"#)?;
    writeln!(f, r#"        }},"#)?;
    writeln!(f, r#"        {{"#)?;
    writeln!(f, r#"            "name": "(gdb) Launch","#)?;
    writeln!(f, r#"            "type": "cppdbg","#)?;
    writeln!(f, r#"            "request": "launch","#)?;
    writeln!(f, r#"            "program": "${{workspaceFolder}}/main","#)?;
    writeln!(f, r#"            "args": [],"#)?;
    writeln!(f, r#"            "stopAtEntry": false,"#)?;
    writeln!(f, r#"            "cwd": "${{workspaceFolder}}","#)?;
    writeln!(f, r#"            "environment": [],"#)?;
    writeln!(f, r#"            "externalConsole": true,"#)?;
    writeln!(f, r#"            "MIMode": "gdb","#)?;
    writeln!(
        f,
        r#"            "preLaunchTask": "procon-assistant compile","#
    )?;
    writeln!(f, r#"            "setupCommands": ["#)?;
    writeln!(f, r#"                {{"#)?;
    writeln!(
        f,
        r#"                    "description": "Enable pretty-printing for gdb","#
    )?;
    writeln!(
        f,
        r#"                    "text": "-enable-pretty-printing","#
    )?;
    writeln!(f, r#"                    "ignoreFailures": true"#)?;
    writeln!(f, r#"                }}"#)?;
    writeln!(f, r#"            ]"#)?;
    writeln!(f, r#"        }}"#)?;
    writeln!(f, r#"    ]"#)?;
    writeln!(f, r#"}}"#)?;
    Ok(())
}

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config()?;
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

    let p = ensure_not_exists(".vscode/tasks.json")?;
    generate_vscode_tasks_json(p)
        .map_err(|e| Error::with_cause("generating .vscode/tasks.json", "failed to write", box e))?;
    print_generated!(".vscode/tasks.json");

    let p = ensure_not_exists(".vscode/launch.json")?;
    generate_vscode_launch_json(p).map_err(|e| {
        Error::with_cause("generating .vscode/launch.json", "failed to write", box e)
    })?;
    print_generated!(".vscode/launch.json");

    if config.auto_open {
        match config.open_directory_instead_of_specific_file {
            true => common::open(&config.editor, ".")?,
            false => common::open(&config.editor, "main.cpp")?,
        }
    }

    Ok(())
}
