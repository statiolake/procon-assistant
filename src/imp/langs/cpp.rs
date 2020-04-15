use super::Lang;
use super::{FilesToOpen, Preprocessed, Progress, RawSource};
use crate::imp::config::MinifyMode;
use crate::imp::config::CONFIG;
use crate::imp::fs as impfs;
use crate::imp::process;
use crate::{eprintln_debug, eprintln_debug_more};
use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs as stdfs, mem};
use walkdir::WalkDir;

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

pub struct Cpp;

impl Lang for Cpp {
    fn check() -> bool {
        Path::new("main.cpp").exists()
    }

    fn new_boxed() -> Box<dyn Lang>
    where
        Self: Sized,
    {
        Box::new(Cpp)
    }

    fn lang_name() -> &'static str
    where
        Self: Sized,
    {
        "cpp"
    }

    fn get_source(&self) -> Result<RawSource> {
        stdfs::read_to_string(Path::new("main.cpp"))
            .map(RawSource)
            .map_err(Into::into)
    }

    fn init_async(&self, path: &Path) -> Progress<anyhow::Result<()>> {
        let path_project = path.to_path_buf();
        Progress::from_fn(move |sender| {
            let template_dir = &CONFIG.langs.cpp.template_dir;

            let _ = sender.send("creating project directory".into());
            create_project_directory(&path_project)?;

            for entry in WalkDir::new(template_dir).min_depth(1) {
                let entry = entry.context("reading template directory failed")?;

                let is_file = entry
                    .metadata()
                    .with_context(|| {
                        format!("getting metadata for `{}` failed", entry.path().display())
                    })?
                    .is_file();

                if !is_file {
                    continue;
                }

                let path = entry.path().strip_prefix(template_dir).with_context(|| {
                    format!(
                        "failed to remove prefix `{}` from `{}`",
                        template_dir.display(),
                        entry.path().display()
                    )
                })?;

                let _ = sender.send(format!("generating `{}`", path.display()));
                safe_generate(&path_project, path)?;
            }

            Ok(())
        })
    }

    fn to_open(&self, path: &Path) -> FilesToOpen {
        FilesToOpen {
            files: vec![path.join("main.cpp")],
            directory: path.to_path_buf(),
        }
    }

    fn open_docs(&self) -> Result<()> {
        process::open_browser("https://cpprefjp.github.io/").context("failed to start browser")?;

        Ok(())
    }

    fn needs_compile(&self) -> bool {
        let target = if cfg!(windows) { "main.exe" } else { "main" };
        impfs::cmp_modified_time("main.cpp", target)
            .map(|ord| ord == Ordering::Greater)
            .unwrap_or(true)
    }

    fn compile_command(&self) -> Vec<Command> {
        let mut cmd = Command::new("clang++");
        let libdir = libdir().display().to_string();
        cmd.arg(format!("-I{}", libdir.escape_default()));
        cmd.args(&[
            "-g",
            #[cfg(windows)]
            "-gcodeview",
            "-O0",
            #[cfg(unix)]
            "-fdiagnostics-color=always",
            #[cfg(unix)]
            "-fsanitize=address,leak,undefined",
            #[cfg(windows)]
            "-Xclang",
            #[cfg(windows)]
            "-flto-visibility-public-std",
            #[cfg(windows)]
            "-fno-delayed-template-parsing",
            "-std=c++14",
            "-Wall",
            "-Wextra",
            "-Wno-old-style-cast",
            "-DPA_DEBUG",
            #[cfg(unix)]
            "-omain",
            #[cfg(windows)]
            "-omain.exe",
            "main.cpp",
            "-fdiagnostics-color=always",
        ]);

        vec![cmd]
    }

    fn run_command(&self) -> Command {
        if cfg!(windows) {
            Command::new(r#".\main.exe"#)
        } else {
            Command::new("./main")
        }
    }

    fn preprocess(&self, source: &RawSource, minify: MinifyMode) -> Result<Preprocessed> {
        let pped = parse_include(&libdir(), &mut HashSet::new(), source, minify, 0)?;

        Ok(Preprocessed(pped))
    }

    fn lint(&self, source: &RawSource) -> Result<Vec<String>> {
        let Preprocessed(pped) = self
            .preprocess(source, MinifyMode::All)
            .context("failed to preprocess the raw source")?;
        let mut result = Vec::new();
        if pped.contains("cerr") {
            result.push("cerr found".into());
        }

        Ok(result)
    }
}

fn libdir() -> PathBuf {
    let mut home_dir = impfs::get_home_path();
    home_dir.push("procon-lib");
    home_dir
}

fn create_project_directory(path_project: &Path) -> Result<()> {
    stdfs::create_dir_all(path_project).with_context(|| {
        format!(
            "failed to create destination directories at `{}`",
            path_project.display()
        )
    })
}

fn safe_generate(path_project: &Path, path: &Path) -> Result<()> {
    if path_project.join(path).exists() {
        eprintln_debug!("file {} already exists, skipping", path.display());
        return Ok(());
    }

    generate(path_project, path)?;

    Ok(())
}

fn generate(path_project_root: &Path, path: &Path) -> Result<()> {
    let path_template = CONFIG.langs.cpp.template_dir.join(path);
    let path_project = path_project_root.join(path);

    eprintln_debug!("loading template from `{}`", path_template.display());

    let template = stdfs::read_to_string(&path_template).with_context(|| {
        format!(
            "failed to read template directory `{}`",
            path_template.display()
        )
    })?;

    let abs_path_project_root = to_absolute::to_absolute_from_current_dir(path_project_root)
        .with_context(|| {
            format!(
                "get absolute path for `{}` failed",
                path_project_root.display()
            )
        })?;
    let template = template
        .replace("$LIB_DIR", &libdir_escaped())
        .replace("$GDB_PATH", &gdbpath_escaped())
        .replace("$PROJECT_PATH", &escape_path(abs_path_project_root));

    write_file_ensure_parent_dirs(&path_project, &template)
}

fn write_file_ensure_parent_dirs(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        stdfs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create destination directories at `{}`",
                parent.display()
            )
        })?;
    }

    stdfs::write(path, contents).with_context(|| format!("failed to write to `{}`", path.display()))
}

fn gdbpath_escaped() -> String {
    which::which("gdb")
        .map(escape_path)
        .unwrap_or_else(|_| "dummy - could not find GDB in your system".into())
}

fn libdir_escaped() -> String {
    escape_path(libdir())
}

fn escape_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .display()
        .to_string()
        .escape_default()
        .to_string()
}

fn parse_include(
    lib_dir: &Path,
    included: &mut HashSet<PathBuf>,
    RawSource(source): &RawSource,
    mode: MinifyMode,
    depth: usize,
) -> Result<String> {
    assert!(lib_dir.is_absolute());

    let mut lines = source.lines().map(ToString::to_string).collect_vec();

    for line in lines.iter_mut() {
        let inc_path: PathBuf = match RE_INCLUDE.captures(&line) {
            None => continue,
            Some(caps) => {
                let inc_file = caps.name("inc_file").unwrap().as_str();
                let inc_path = lib_dir.join(Path::new(inc_file));
                let inc_path: PathBuf = inc_path.components().collect();
                to_absolute::canonicalize(&inc_path)
                    .with_context(|| format!("failed to canonicalize `{}`", inc_path.display()))?
            }
        };

        eprintln_debug!("including {}", inc_path.display());
        let will_be_replaced = if included.contains(&inc_path) {
            eprintln_debug_more!(
                "... skipping previously included file {}",
                inc_path.display()
            );

            String::new()
        } else {
            included.insert(inc_path.clone());
            let next_lib_dir = inc_path
                .parent()
                .expect("internal error: cannot extract parent");

            let source = stdfs::read_to_string(&inc_path)
                .with_context(|| format!("failed to read `{}`", inc_path.display()))?;

            parse_include(next_lib_dir, included, &RawSource(source), mode, depth + 1)?
        };

        mem::replace(line, will_be_replaced);
    }
    let modified = lines.join("\n");

    match (mode, depth) {
        (MinifyMode::All, 0) => minify(&modified),
        (MinifyMode::TemplateOnly, 1) => minify(&modified),
        _ => Ok(modified),
    }
}

fn remove_block_comments(content: &str) -> String {
    RE_BLOCK_COMMENT.replace_all(content, "").into()
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

fn minify(source: &str) -> Result<String> {
    let source = remove_block_comments(source);
    let lines: Vec<String> = source.split('\n').map(|x| x.into()).collect();
    let comment_removed = remove_line_comments(lines);
    let debug_statement_removed = remove_debug_statement(comment_removed);
    let removed = remove_pragma_once(debug_statement_removed);

    let mut result = removed
        .into_iter()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect_vec();

    let replaces = [
        (&*RE_WHITESPACE_AFTER_BLOCK_COMMENT, "*/"),
        (&*RE_WHITESPACE_AFTER_COLONS, "$col"),
        (&*RE_MULTIPLE_SPACE, " "),
        (&*RE_WHITESPACE_AROUND_OPERATOR, "$op"),
        (&*RE_WHITESPACE_AROUND_PAREN, "$par"),
    ];

    for &(regex, replace) in replaces.iter() {
        for line in &mut result {
            let replaced = regex.replace_all(line, replace);
            let replaced = replaced.trim();
            *line = replaced.to_string();
        }
    }

    Ok(concat_safe_lines(result).join("\n"))
}
