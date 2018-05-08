use clipboard::{ClipboardContext, ClipboardProvider};

use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::Path;

use regex::Regex;

use common;
use config;
use {Error, Result};

pub fn copy_to_clipboard(file_path: &Path) -> Result<()> {
    print_copying!("{} to clipboard", file_path.display());
    let main_src = read_source_file(file_path)?;
    let mut provider: ClipboardContext = ClipboardProvider::new()
        .map_err(|_| Error::new("copying to clipboard", "cannot get clipboard provider"))?;
    provider.set_contents(main_src).map_err(|_| {
        Error::new(
            "copying to clipboard",
            "failed to set contents to clipboard.",
        )
    })?;
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
    parse_include(file_path, src_content)
}

fn parse_include(curr_file_path: &Path, content: String) -> Result<String> {
    let re_inc = Regex::new(r#" *# *include *" *([^>]*) *""#).unwrap();
    let lib_dir = if config::HEADER_FILE_EXTENSIONS.contains(&curr_file_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap())
    {
        curr_file_path.parent().unwrap().to_path_buf()
    } else {
        common::get_procon_lib_dir()?
    };
    let mut modified_content: Vec<String> = content.split('\n').map(|x| x.to_string()).collect();
    for line in modified_content.iter_mut() {
        for cap in re_inc.captures_iter(&line.clone()) {
            let inc_file = &cap[1];
            let inc_path = format!("{}/{}", lib_dir.display(), inc_file);
            print_including!("{}", inc_path);
            let inc_src = read_source_file(inc_path.as_ref())?;
            let replaced = re_inc.replace(line, &*inc_src).to_string();
            mem::replace(line, replaced);
        }
    }
    let modified_content = modified_content.join("\n");

    Ok(modified_content)
}
