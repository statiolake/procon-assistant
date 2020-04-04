use super::{FilesToOpen, Preprocessed, RawSource};
use super::{Language, Progress};
use crate::eprintln_debug;
use crate::imp::config::MinifyMode;
use crate::imp::config::RustProjectTemplate;
use crate::ui::CONFIG;
use anyhow::{anyhow, ensure};
use anyhow::{Context, Result};
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use scopefunc::ScopeFunc;
use scopeguard::defer;
use std::env;
use std::fs as stdfs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Rust;

lazy_static! {
    static ref RE_MOD_PATH: Regex = Regex::new(r#"#\[path\s+=\s+"(?P<path>.*)"\]"#).unwrap();
    static ref RE_MOD: Regex = Regex::new(r#"(?:pub\s+)?mod (?P<name>\w+);"#).unwrap();
    static ref RE_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();
}

impl Language for Rust {
    fn check() -> bool {
        Path::new("main/Cargo.toml").exists()
    }

    fn new_boxed() -> Box<dyn Language>
    where
        Self: Sized,
    {
        Box::new(Rust)
    }

    fn language_name() -> &'static str
    where
        Self: Sized,
    {
        "rust"
    }

    fn init_async(&self, path: &Path) -> Progress<Result<FilesToOpen>> {
        let path = path.to_path_buf();
        Progress::from_fn(move |sender| {
            let cwd = env::current_dir()?;

            let _ = sender.send("creating project directory".into());
            create_project_directory(&path)?;

            // set current directory to the created project directory
            env::set_current_dir(&path)?;

            // restore the original current directory after finish
            defer! {
                // to use `defer!`, we need to ignore the error
                let _ = env::set_current_dir(cwd);
            }

            let _ = sender.send("generating new cargo project".into());
            // generate a project
            match &CONFIG.languages.rust.project_template {
                RustProjectTemplate::Git { repository, branch } => generate_git(repository, branch),
                RustProjectTemplate::Local { path } => generate_local(path),
            }
            .context("failed to generate a project")?;

            let _ = sender.send("building generated project".into());
            // pre-build the project
            let output = Command::new("cargo")
                .arg("build")
                .arg("--quiet")
                .arg("--manifest-path")
                .arg("main/Cargo.toml")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()?
                .wait_with_output()?;
            ensure!(
                output.status.success(),
                "failed to build a project: {}",
                String::from_utf8_lossy(&output.stderr)
            );

            Ok(FilesToOpen {
                files: vec![path.join("main").join("src").join("main.rs")],
                directory: path.join("main"),
            })
        })
    }

    fn get_source(&self) -> Result<RawSource> {
        stdfs::read_to_string("main/src/main.rs")
            .map(RawSource)
            .map_err(Into::into)
    }

    fn needs_compile(&self) -> bool {
        // in Rust, to avoid copying a large `target` directory, `target`
        // directory is symlinked to the template directory. This means that the
        // binary is placed in the same place for all projects. It causes the
        // binary overwritten by another project. To prevent running wrong
        // binary, we always need to clean the binary and compile.
        true
    }

    fn compile_command(&self) -> Vec<Command> {
        let clean = Command::new("cargo").modify(|c| {
            c.arg("clean")
                .arg("--manifest-path")
                .arg("main/Cargo.toml")
                .arg("-p")
                .arg("main");
        });

        let build = Command::new("cargo").modify(|c| {
            c.arg("build").arg("--manifest-path").arg("main/Cargo.toml");
        });

        vec![clean, build]
    }

    fn run_command(&self) -> Command {
        Command::new("cargo").modify(|c| {
            c.arg("run")
                .arg("-q")
                .arg("--manifest-path")
                .arg("main/Cargo.toml");
        })
    }

    fn preprocess(
        &self,
        RawSource(source): &RawSource,
        minify: MinifyMode,
    ) -> Result<Preprocessed> {
        let source = resolve_mod(Path::new("main/src"), source.clone(), minify, 0)?;

        Ok(Preprocessed(source))
    }

    fn lint(&self, _pped: &Preprocessed) -> Vec<String> {
        // TODO: implement
        vec![]
    }
}

fn create_project_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn generate_git(repository: &str, branch: &str) -> Result<()> {
    let output = Command::new("cargo")
        .arg("generate")
        .arg("--git")
        .arg(repository)
        .arg("--branch")
        .arg(branch)
        .arg("--name")
        .arg("main")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

fn generate_local(path: &Path) -> Result<()> {
    eprintln_debug!("copying from `{}`", path.display());

    let options = CopyOptions {
        skip_exist: true,
        copy_inside: true,
        ..CopyOptions::new()
    };

    let base_path = Path::new("main");
    stdfs::create_dir(base_path)?;
    for entry in stdfs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry_path.file_name().unwrap();
        let entry_metadata = entry.metadata()?;
        if entry_metadata.is_file() {
            stdfs::copy(&entry_path, base_path.join(entry_name))?;
        } else if entry_metadata.is_dir() && entry_name != "target" {
            dir::copy(&entry_path, base_path.join(entry_name), &options)?;
        }
    }

    // symlink target directory
    let template_target_path = path.join("target");
    let project_target_path = base_path.join("target");
    if template_target_path.exists() {
        symlink::symlink_dir(template_target_path, project_target_path)?;
    }

    Ok(())
}

fn resolve_mod(cwd: &Path, source: String, mode: MinifyMode, depth: usize) -> Result<String> {
    let mut result = Vec::new();
    let mut path_attr = None;
    for line in source.lines() {
        if line.trim().is_empty() {
            result.push("".to_string());
            continue;
        }

        match RE_MOD.captures(&line) {
            None => {
                match RE_MOD_PATH.captures(&line) {
                    Some(caps) => path_attr = Some(caps.name("path").unwrap().as_str().to_string()),
                    None => {
                        path_attr = None;
                        result.push(RE_COMMENT.replace_all(line, "").to_string());
                    }
                }
                continue;
            }
            Some(caps) => {
                let mod_name = caps.name("name").unwrap().as_str().to_string();
                let mod_path = match path_attr {
                    Some(path) => {
                        let mut path = PathBuf::from(path);
                        if !path.is_absolute() {
                            path = cwd.join(path);
                        }
                        path
                    }
                    None => {
                        let file = cwd.join(format!("{}.rs", mod_name));
                        let dir = cwd.join(format!("{}/mod.rs", mod_name));
                        eprintln_debug!("searching file: {}", file.display());
                        eprintln_debug!("searching dir: {}", dir.display());

                        if file.exists() {
                            file
                        } else if dir.exists() {
                            dir
                        } else {
                            panic!("failed to find the module");
                        }
                    }
                };
                let source = stdfs::read_to_string(&mod_path)?;
                let next_cwd = cwd.join(&mod_name);
                let replaced = resolve_mod(&next_cwd, source, mode, depth + 1)?;
                result.push(format!("mod {} {{\n{}\n}}", mod_name, replaced));

                path_attr = None;
            }
        };
    }

    let mut pped = result.join("\n");
    if !(mode == MinifyMode::TemplateOnly && depth == 0) {
        if let Ok(fmted) = rustfmt(&pped) {
            pped = fmted;
        }
    }

    match (mode, depth) {
        (MinifyMode::All, 0) => minify(&pped),
        (MinifyMode::TemplateOnly, 1) => minify(&pped),
        _ => Ok(pped),
    }
}

fn rustfmt(source: &str) -> Result<String> {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to get stdin"))?;
    stdin.write_all(source.as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn minify(source: &str) -> Result<String> {
    Ok(source.lines().map(|x| x.trim().to_string()).join(""))
}
