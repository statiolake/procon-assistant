use std::collections::HashSet;
use std::mem;
use std::path::{Path, PathBuf};

use regex::Regex;

use imp::config;
use imp::langs;

use super::{Error, ErrorKind, Result};
use super::{Minified, Preprocessed, RawSource};

pub fn preprocess(content: RawSource) -> Result<Preprocessed> {
    let content = parse_include(None, content)?;
    let content = remove_block_comments(content);
    let lines: Vec<String> = content.split('\n').map(|x| x.into()).collect();
    let removed = Some(lines)
        .map(remove_line_comments)
        .map(remove_include_guard)
        .expect("logical error: this must be Some because nothing change it None.")?;
    Ok(Preprocessed(removed))
}

pub fn minify(preprocessed_lines: Preprocessed) -> Minified {
    let mut result = preprocessed_lines.into_inner();
    result = result
        .into_iter()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
    let re_whitespace_after_block_comment = Regex::new(r#"\*/\s+"#).unwrap();
    let re_whitespace_after_colons = Regex::new(r#"\s*(?P<col>[;:])\s*"#).unwrap();
    let re_multiple_space = Regex::new(r#"\s+"#).unwrap();
    let re_whitespace_around_operator =
        Regex::new(r#"\s*(?P<op>[+\-*/%~^|&<>=,.!?]|<<|>>|<=|>=|==|!=|\+=|-=|\*=|/=)\s*"#).unwrap();
    let re_whitespace_around_paren = Regex::new(r#"\s*(?P<par>[({)}])\s*"#).unwrap();

    for (regex, replace) in [
        (re_whitespace_after_block_comment, "*/"),
        (re_whitespace_after_colons, "$col"),
        (re_multiple_space, " "),
        (re_whitespace_around_operator, "$op"),
        (re_whitespace_around_paren, "$par"),
    ]
        .iter()
    {
        for r in &mut result {
            *r = r.trim().into();
            *r = regex.replace_all(r, replace as &str).into();
        }
    }
    Minified(result.join("\n"))
}

fn parse_include(curr_file_path: Option<&Path>, content: RawSource) -> Result<String> {
    let content = content.into_inner();
    let re_inc = Regex::new(r#" *# *include *" *([^>]*) *""#).unwrap();
    let curr_extension = curr_file_path.map(|path| {
        path.extension()
            .unwrap()
            .to_str()
            .expect("critical error: failed to get file extension.")
    });
    let is_header = curr_extension
        .map(|ext| config::HEADER_FILE_EXTENSIONS.contains(&ext))
        .unwrap_or(false);

    let lib_dir = if is_header {
        curr_file_path
            .expect("logical error: this must be some")
            .parent()
            .unwrap()
            .to_path_buf()
    } else {
        (langs::cpp::LANG.lib_dir_getter)()
    };

    let mut modified_content: Vec<String> = content.split('\n').map(|x| x.to_string()).collect();
    for line in modified_content.iter_mut() {
        for cap in re_inc.captures_iter(&line.clone()) {
            let inc_file = &cap[1];
            let inc_path = lib_dir.join(Path::new(inc_file).components().collect::<PathBuf>());
            print_info!(true, "including {}", inc_path.display());
            let inc_src = super::read_source_file(&inc_path)
                .and_then(|src| parse_include(Some(&inc_path), src))?;
            let replaced = re_inc.replace(line, &*inc_src).to_string();
            mem::replace(line, replaced);
        }
    }
    let modified_content = modified_content.join("\n");

    Ok(modified_content)
}

fn remove_block_comments(content: String) -> String {
    let re_block_comment = Regex::new(r#"(?s)/\*.*?\*/"#).unwrap();
    re_block_comment.replace_all(&content, "").into()
}

fn remove_line_comments(mut lines: Vec<String>) -> Vec<String> {
    let re_line_comment = Regex::new(r#"//.*"#).unwrap();

    for line in &mut lines {
        *line = re_line_comment.replace_all(line, "").trim().into();
    }
    lines
}

fn remove_include_guard(mut lines: Vec<String>) -> Result<Vec<String>> {
    let re_preprocessor_directive = Regex::new(r#"\s*#.*"#).unwrap();
    let re_continuing_backslash = Regex::new(r#"\\\s*$"#).unwrap();
    let re_if = Regex::new(r#"\s*#\s*if"#).unwrap();
    let re_ifndef = Regex::new(r#"\s*#\s*ifndef\s*(.*)"#).unwrap();
    let re_define = Regex::new(r#"\s*#\s*define\s*([^(]*)"#).unwrap();
    let re_endif = Regex::new(r#"\s*#\s*endif"#).unwrap();

    let mut result = Vec::new();
    let mut building = String::new();
    let mut defined = HashSet::new();
    let mut i = 0;

    while i < lines.len() {
        let find_corresponding_endif =
            |lines: &Vec<String>, line_of_ifdef: usize| -> Result<usize> {
                let mut curr = line_of_ifdef + 1;
                let mut if_nest = 1;
                while if_nest > 0 {
                    if curr >= lines.len() {
                        return Err(Error::new(ErrorKind::PreProcessorIfNotMatch()));
                    }

                    if re_if.is_match(&lines[curr]) {
                        if_nest += 1;
                    } else if re_endif.is_match(&lines[curr]) {
                        if_nest -= 1;
                    }
                    curr += 1;
                }
                Ok(curr - 1)
            };

        if re_preprocessor_directive.is_match(&lines[i]) {
            result.push(mem::replace(&mut building, String::new()));
            let has_enough_space_for_ifdef_define = i < lines.len() - 1;
            if has_enough_space_for_ifdef_define
                && re_ifndef.is_match(&lines[i])
                && re_define.is_match(&lines[i + 1])
            {
                let symbol_name = re_ifndef
                    .captures(&lines[i])
                    .expect("there must be ifndef!")[1]
                    .to_string();
                if defined.contains(&symbol_name) {
                    i = find_corresponding_endif(&lines, i)?;
                } else {
                    defined.insert(symbol_name);
                    let j = find_corresponding_endif(&lines, i)?;
                    lines.remove(j);
                    i += 1;
                }
            } else {
                loop {
                    result.push(lines[i].clone());
                    if !re_continuing_backslash.is_match(&lines[i]) {
                        break;
                    }
                    i += 1
                }
            }
        } else {
            building += &lines[i];
        }
        building += " ";

        i += 1;
    }
    if !building.is_empty() {
        result.push(building);
    }
    Ok(result)
}
