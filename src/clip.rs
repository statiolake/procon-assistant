use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::process::{Command, Stdio};

use regex::Regex;

use common;
use run;
use run::SrcFile;

use Error;
use Result;

pub fn main() -> Result<()> {
    let SrcFile { file_name, .. } = run::get_source_file()?;
    copy_to_clipboard(file_name.as_ref())
}

pub fn copy_to_clipboard(file_path: &Path) -> Result<()> {
    print_copying!("{} to clipboard", file_path.display());
    let main_src = read_source_file(file_path)?;
    let modified = parse_include(main_src)?;

    let resultchild = Command::new("xsel")
        .arg("-b")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(mut child) = resultchild {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(modified.as_bytes())
            .unwrap();
        child.wait().unwrap();
    }
    print_finished!("copying");
    Ok(())
}

fn read_source_file(file_path: &Path) -> Result<String> {
    let mut src_content = String::new();
    File::open(file_path)
        .map_err(|e| {
            Error::with_cause(
                format!("loading {}", file_path.display()),
                "failed to open the specified file.",
                box e,
            )
        })?
        .read_to_string(&mut src_content)
        .map_err(|e| {
            Error::with_cause(
                format!("loading {}", file_path.display()),
                "failed to read the entire content of the file.",
                box e,
            )
        })?;
    parse_include(src_content)
}

fn parse_include(content: String) -> Result<String> {
    let re_inc = Regex::new(r#" *# *include *" *([^>]*) *""#).unwrap();
    let lib_dir = common::get_procon_lib_dir()?;
    let mut modified_content: Vec<String> = content.split('\n').map(|x| x.to_string()).collect();
    let mut modified = false;
    for line in modified_content.iter_mut() {
        for cap in re_inc.captures_iter(&line.clone()) {
            modified = true;
            let inc_file = &cap[1];
            let inc_path = format!("{}/{}", lib_dir, inc_file);
            print_including!("{}", inc_path);
            let inc_src = read_source_file(inc_path.as_ref())?;
            let replaced = re_inc.replace(line, &*inc_src).to_string();
            mem::replace(line, replaced);
        }
    }
    let modified_content = modified_content.join("\n");

    if modified {
        parse_include(modified_content)
    } else {
        Ok(modified_content)
    }
}
