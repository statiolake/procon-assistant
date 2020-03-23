use super::{Error, ErrorKind, Minified, Preprocessed, RawSource, Result};
use crate::eprintln_info;
use crate::imp::langs;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::mem;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref RE_INCLUDE: Regex =
        Regex::new(r#"\s*#\s*include\s*"(?P<inc_file>[^"]*)\s*""#).unwrap();
    static ref RE_PRAGMA_ONCE: Regex = Regex::new(r#"\s*#\s*pragma\s+once\s*"#).unwrap();
    static ref RE_DEBUG_STATEMENT: Regex = Regex::new(r#"\s*PD\s*\((?P<stmt>[^;]*)\);"#).unwrap();
    static ref RE_WHITESPACE_AFTER_BLOCK_COMMENT: Regex = Regex::new(r#"\*/\s+"#).unwrap();
    static ref RE_WHITESPACE_AFTER_COLONS: Regex = Regex::new(r#"\s*(?P<col>[;:])\s*"#).unwrap();
    static ref RE_MULTIPLE_SPACE: Regex = Regex::new(r#"\s+"#).unwrap();
    static ref RE_WHITESPACE_AROUND_OPERATOR: Regex =
        Regex::new(r#"\s*(?P<op>[+\-*/%~^|&<>=,.!?]|<<|>>|<=|>=|==|!=|\+=|-=|\*=|/=)\s*"#).unwrap();
    static ref RE_WHITESPACE_AROUND_PAREN: Regex = Regex::new(r#"\s*(?P<par>[({)}])\s*"#).unwrap();
    static ref RE_BLOCK_COMMENT: Regex = Regex::new(r#"(?s)/\*.*?\*/"#).unwrap();
    static ref RE_LINE_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();
}

pub fn preprocess(quiet: bool, content: RawSource) -> Result<Preprocessed> {
    let content = parse_include(
        quiet,
        &(langs::cpp::LANG.lib_dir_getter)(),
        &mut HashSet::new(),
        content,
    )?;
    let content = remove_block_comments(content);
    let lines: Vec<String> = content.split('\n').map(|x| x.into()).collect();
    let comment_removed = remove_line_comments(lines);
    let debug_statement_removed = remove_debug_statement(comment_removed);
    let removed = remove_pragma_once(debug_statement_removed);
    Ok(Preprocessed(concat_safe_lines(removed)))
}

pub fn minify(_quiet: bool, preprocessed_lines: Preprocessed) -> Minified {
    let mut result = preprocessed_lines.into_inner();
    result = result
        .into_iter()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();

    let replaces = [
        (&*RE_WHITESPACE_AFTER_BLOCK_COMMENT, "*/"),
        (&*RE_WHITESPACE_AFTER_COLONS, "$col"),
        (&*RE_MULTIPLE_SPACE, " "),
        (&*RE_WHITESPACE_AROUND_OPERATOR, "$op"),
        (&*RE_WHITESPACE_AROUND_PAREN, "$par"),
    ];

    for (regex, replace) in replaces.iter() {
        for r in &mut result {
            *r = r.trim().into();
            *r = regex.replace_all(r, replace as &str).into();
        }
    }
    Minified(result.join("\n"))
}

fn parse_include(
    quiet: bool,
    lib_dir: &Path,
    included: &mut HashSet<PathBuf>,
    content: RawSource,
) -> Result<String> {
    let content = content.into_inner();
    assert!(lib_dir.is_absolute());

    let mut lines: Vec<String> = content.split('\n').map(ToString::to_string).collect();

    for line in lines.iter_mut() {
        let inc_path: PathBuf = match RE_INCLUDE.captures(&line) {
            None => continue,
            Some(caps) => {
                let inc_file = caps.name("inc_file").unwrap().as_str();
                let inc_path = lib_dir.join(Path::new(inc_file));
                let inc_path: PathBuf = inc_path.components().collect();
                to_absolute::canonicalize(&inc_path).map_err(|e| {
                    Error(ErrorKind::CanonicalizationFailed {
                        source: e.into(),
                        path: inc_path,
                    })
                })?
            }
        };

        if !quiet {
            eprintln_info!("including {}", inc_path.display());
        }
        let will_be_replaced = if included.contains(&inc_path) {
            if !quiet {
                eprintln_info!(
                    "... skipping previously included file {}",
                    inc_path.display()
                );
            }
            String::new()
        } else {
            included.insert(inc_path.clone());
            let next_lib_dir = inc_path
                .parent()
                .expect("internal error: cannot extract parent");
            super::read_source_file(&inc_path)
                .and_then(|src| parse_include(quiet, next_lib_dir, included, src))?
        };

        mem::replace(line, will_be_replaced);
    }
    let modified_content = lines.join("\n");

    Ok(modified_content)
}

fn remove_block_comments(content: String) -> String {
    RE_BLOCK_COMMENT.replace_all(&content, "").into()
}

fn remove_line_comments(mut lines: Vec<String>) -> Vec<String> {
    for line in &mut lines {
        *line = RE_LINE_COMMENT.replace_all(line, "").trim().into();
    }
    lines
}

fn remove_debug_statement(mut lines: Vec<String>) -> Vec<String> {
    for line in &mut lines {
        *line = RE_DEBUG_STATEMENT.replace_all(line, "").trim().into();
    }
    lines
}

fn remove_pragma_once(mut lines: Vec<String>) -> Vec<String> {
    for line in &mut lines {
        *line = RE_PRAGMA_ONCE.replace_all(line, "").trim().into();
    }
    lines
}

fn concat_safe_lines(lines: Vec<String>) -> Vec<String> {
    fn push_and_init(vec: &mut Vec<String>, line: &mut String) {
        if !line.is_empty() {
            vec.push(mem::replace(line, String::new()));
        }
    }

    let mut res = Vec::new();
    let mut res_line = String::new();

    let mut line_continues;
    for line in lines {
        let line = line.trim();
        line_continues = true;

        if line.starts_with('#') {
            // flush current string
            push_and_init(&mut res, &mut res_line);
            line_continues = line.ends_with('\\');
        }

        if res_line != "" {
            // to avoid concatenating two tokens (to avoid previously separated
            // by newline character to be concatenated because of elision of
            // newline charactor), insert space between two lines.
            res_line += " ";
        }

        res_line += line.trim_matches('\\').trim();

        if !line_continues {
            push_and_init(&mut res, &mut res_line);
        }
    }

    // push last line if something left
    push_and_init(&mut res, &mut res_line);

    res
}
