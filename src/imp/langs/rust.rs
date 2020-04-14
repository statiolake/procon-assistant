use super::{FilesToOpen, Preprocessed, RawSource};
use super::{Lang, Progress};
use crate::eprintln_debug;
use crate::imp::config::MinifyMode;
use crate::imp::config::RustProjectTemplate;
use crate::imp::config::CONFIG;
use crate::imp::process;
use anyhow::{anyhow, bail, ensure};
use anyhow::{Context, Result};
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use itertools::Itertools;
use lazy_static::lazy_static;
use quote::ToTokens;
use regex::Regex;
use scopefunc::ScopeFunc;
use scopeguard::defer;
use std::env;
use std::fs as stdfs;
use std::io::prelude::*;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::{Command, Stdio};

pub struct Rust;

lazy_static! {
    static ref RE_MOD_PATH: Regex = Regex::new(r#"#\[path\s+=\s+"(?P<path>.*)"\]"#).unwrap();
    static ref RE_MOD: Regex = Regex::new(r#"\bmod\s+(?P<name>\w+);"#).unwrap();
    static ref RE_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();
    static ref RE_WHITESPACE_AFTER_COLONS: Regex = Regex::new(r#"\s*(?P<col>[;:])\s*"#).unwrap();
    static ref RE_MULTIPLE_SPACE: Regex = Regex::new(r#"\s+"#).unwrap();
    static ref RE_WHITESPACE_AROUND_OPERATOR: Regex =
        Regex::new(r#"\s*(?P<op>[+\-*/%~^|&<>=,.!?\[\]]|<<|>>|<=|>=|==|!=|\+=|-=|\*=|/=|->)\s*"#)
            .unwrap();
    static ref RE_WHITESPACE_AROUND_PAREN: Regex = Regex::new(r#"\s*(?P<par>[({)}])\s*"#).unwrap();
}

impl Lang for Rust {
    fn check() -> bool {
        Path::new("main/Cargo.toml").exists()
    }

    fn new_boxed() -> Box<dyn Lang>
    where
        Self: Sized,
    {
        Box::new(Rust)
    }

    fn lang_name() -> &'static str
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
            match &CONFIG.langs.rust.project_template {
                RustProjectTemplate::Git { repository, branch } => generate_git(repository, branch),
                RustProjectTemplate::Local { path } => generate_local(path),
            }
            .context("failed to generate a project")?;

            let _ = sender.send("building generated project".into());
            // pre-build the project
            let output = Command::new("cargo")
                .arg("build")
                .arg("--quiet")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .current_dir("main")
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

    fn open_docs(&self) -> Result<()> {
        // open crate docs
        let path = to_absolute::to_absolute_from_current_dir("main/target/doc/main/index.html")
            .context("failed to get the absolute path for the document")?;
        let path_url_base = path.display().to_string().replace(MAIN_SEPARATOR, "/");
        let crate_docs = format!("file:///{}", path_url_base);
        process::open_browser(&crate_docs).context("failed to open crate docs")?;

        // open std docs
        Command::new("rustup")
            .arg("doc")
            .arg("--std")
            .spawn()
            .context("failed to open std doc")?;

        Ok(())
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
            c.arg("clean").arg("-p").arg("main").current_dir("main");
        });

        let build = Command::new("cargo").modify(|c| {
            c.arg("build").current_dir("main");
        });

        vec![clean, build]
    }

    fn run_command(&self) -> Command {
        Command::new("main/target/debug/main").modify(|cmd| {
            cmd.env("RUST_BACKTRACE", "1");
        })
    }

    fn preprocess(
        &self,
        RawSource(source): &RawSource,
        minify: MinifyMode,
    ) -> Result<Preprocessed> {
        let source = expand_source(Path::new("main/src"), &source, minify)?;
        Ok(Preprocessed(source))
    }

    fn lint(&self, source: &RawSource) -> Result<Vec<String>> {
        let Preprocessed(pped) = self
            .preprocess(source, MinifyMode::All)
            .context("failed to preprocess the source")?;

        let mut res = Vec::new();
        if pped.contains("eprintln!") {
            res.push("eprintln! found".to_string());
        }

        Ok(res)
    }
}

fn create_project_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn generate_git(repository: &str, branch: &str) -> Result<()> {
    if Path::new("main").exists() {
        // skip generating everything if main directory exists
        return Ok(());
    }

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
    stdfs::create_dir_all(base_path)?;
    for entry in stdfs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry_path
            .file_name()
            .expect("critical error: file has no name");
        let target = base_path.join(entry_name);
        if target.exists() {
            // skip existing file or folder
            continue;
        }

        let entry_metadata = entry.metadata()?;
        if entry_metadata.is_file() {
            stdfs::copy(&entry_path, &target)?;
        } else if entry_metadata.is_dir() && entry_name != "target" {
            dir::copy(&entry_path, &target, &options)?;
        }
    }

    // symlink target directory
    let template_target_path = path.join("target");
    let project_target_path = base_path.join("target");
    if template_target_path.exists() && !project_target_path.exists() {
        symlink::symlink_dir(template_target_path, project_target_path)?;
    }

    Ok(())
}

fn expand_source(cwd: &Path, source: &str, mode: MinifyMode) -> Result<String> {
    let file = syn::parse_file(&source).context("failed to parse the source code")?;
    let file = expand_file(cwd, file)?;

    match mode {
        MinifyMode::None => rustfmt(&file.into_token_stream().to_string()),
        MinifyMode::All => minify(&file.into_token_stream().to_string()),
        MinifyMode::TemplateOnly => {
            let syn::File {
                shebang,
                attrs,
                items,
            } = file;

            let mut res = String::new();
            if let Some(shebang) = shebang {
                res.push_str(&shebang);
                res.push('\n');
            }

            res.push_str(&minify(
                &attrs
                    .into_iter()
                    .map(|attr| attr.into_token_stream())
                    .collect::<proc_macro2::TokenStream>()
                    .to_string(),
            )?);
            for item in items {
                match item {
                    syn::Item::Mod(imod) => {
                        res.push_str(&minify(&imod.into_token_stream().to_string())?);
                        res.push('\n');
                    }
                    other => res.push_str(&rustfmt(&other.into_token_stream().to_string())?),
                }
            }

            Ok(res)
        }
    }
}

fn expand_file(cwd: &Path, mut file: syn::File) -> Result<syn::File> {
    file.items = file
        .items
        .into_iter()
        .map(|item| match item {
            syn::Item::Mod(imod) => expand_mod(cwd, imod).map(Into::into),
            item => Ok(item),
        })
        .collect::<Result<_>>()?;

    Ok(file)
}

fn expand_mod(cwd: &Path, imod: syn::ItemMod) -> Result<syn::ItemMod> {
    let semi_span = match imod.semi {
        Some(semi) => semi.spans[0],
        None => return Ok(imod),
    };

    let attrs = imod.attrs;
    let mut paths = Vec::new();
    let mut rest_attrs = Vec::new();
    for attr in attrs {
        let meta = match attr.parse_meta() {
            Ok(meta) => meta,
            Err(_) => {
                rest_attrs.push(attr);
                continue;
            }
        };

        match meta {
            syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) if matches!(path.get_ident(), Some(ident) if ident == "path") => {
                paths.push(lit)
            }
            _ => {
                rest_attrs.push(attr);
                continue;
            }
        }
    }

    ensure!(paths.len() <= 1, "multiple paths are specified for module");
    let (path, next_cwd) = match paths.into_iter().next() {
        Some(path) => match path {
            syn::Lit::Str(s) => {
                let path = PathBuf::from(s.value());
                let next_cwd = path
                    .parent()
                    .expect("failed to get parent directory")
                    .to_path_buf();
                (path, next_cwd)
            }
            _ => bail!("invalid value of type for `#[path = ]`"),
        },
        None => {
            let dir = PathBuf::from(imod.ident.to_string());
            let file = PathBuf::from(format!("{}.rs", imod.ident));
            let dirmod = dir.join("mod.rs");
            eprintln_debug!("searching file: {}", file.display());
            eprintln_debug!("searching dir: {}", dir.display());

            if cwd.join(&file).exists() {
                (file, cwd.join(dir))
            } else if cwd.join(&dirmod).exists() {
                (dirmod, cwd.join(dir))
            } else {
                bail!("failed to find the module");
            }
        }
    };

    // load file from the path and parse
    let source = stdfs::read_to_string(&cwd.join(&path)).with_context(|| {
        format!(
            "failed to read next file `{}` in `{}`",
            path.display(),
            cwd.display()
        )
    })?;
    let file = syn::parse_file(&source).context("failed to parse next module file")?;
    let expanded = expand_file(&next_cwd, file).context("failed to expand next module file")?;

    rest_attrs.extend(expanded.attrs);
    Ok(syn::ItemMod {
        attrs: rest_attrs,
        vis: imod.vis,
        mod_token: imod.mod_token,
        ident: imod.ident,
        content: Some((syn::token::Brace { span: semi_span }, expanded.items)),
        semi: None,
    })
}

fn rustfmt(source: &str) -> Result<String> {
    let mut child = Command::new("rustup")
        .arg("run")
        .arg("stable")
        .arg("rustfmt")
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
    let replaces = [
        (&*RE_MULTIPLE_SPACE, " "),
        (&*RE_WHITESPACE_AFTER_COLONS, "$col"),
        (&*RE_WHITESPACE_AROUND_OPERATOR, "$op"),
        (&*RE_WHITESPACE_AROUND_PAREN, "$par"),
    ];

    let mut result = source
        .lines()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .join(" ");
    for &(regex, replace) in replaces.iter() {
        let replaced = regex.replace_all(&result, replace);
        let replaced = replaced.trim();
        result = replaced.to_string();
    }

    Ok(result)
}
