use std::collections::HashSet;
use std::mem;
use std::path::{Path, PathBuf};

use regex::Regex;

use imp::config;
use imp::langs;

use super::Result;
use super::{Minified, Preprocessed, RawSource};

lazy_static! {
    static ref RE_INC: Regex = Regex::new(r#" *# *include *" *([^>]*) *""#).unwrap();
}

pub fn preprocess(content: RawSource) -> Result<Preprocessed> {
    let content = parse_include(None, &mut HashSet::new(), content)?;
    let content = remove_block_comments(content);
    let lines: Vec<String> = content.split('\n').map(|x| x.into()).collect();
    let removed = remove_line_comments(lines);
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

fn parse_include(
    curr_file_path: Option<&Path>,
    included: &mut HashSet<PathBuf>,
    content: RawSource,
) -> Result<String> {
    let content = content.into_inner();
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
    assert!(lib_dir.is_absolute());

    let mut modified_content: Vec<String> = content.split('\n').map(|x| x.to_string()).collect();
    for line in modified_content.iter_mut() {
        for cap in RE_INC.captures_iter(&line.clone()) {
            let inc_file = &cap[1];
            let inc_path = lib_dir.join(Path::new(inc_file).components().collect::<PathBuf>());

            print_info!(true, "including {}", inc_path.display());
            if included.contains(&inc_path) {
                print_info!(
                    true,
                    "skipping previously included file {}",
                    inc_path.display()
                );
                mem::replace(line, String::new());
                continue;
            }

            included.insert(inc_path.clone());
            let inc_src = super::read_source_file(&inc_path)
                .and_then(|src| parse_include(Some(&inc_path), included, src))?;
            let replaced = RE_INC.replace(line, &*inc_src).to_string();

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
