use super::{FilesToOpen, Minified, Preprocessed, RawSource};
use super::{Language, Progress};
use crate::imp::config::RustProjectTemplate;
use crate::imp::fs as impfs;
use crate::ui::CONFIG;
use anyhow::anyhow;
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use scopefunc::ScopeFunc;
use scopeguard::defer;
use std::cmp::Ordering;
use std::env;
use std::fs as stdfs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("creating directory `{}` failed", .path.display())]
    CreateDestinationDirectoryFailed {
        source: anyhow::Error,
        path: PathBuf,
    },

    #[error("generating project failed")]
    GeneratingProjectFailed { source: anyhow::Error },

    #[error("building project failed")]
    BuildingProjectFailed { source: anyhow::Error },
}

pub struct Rust;

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

    fn init_async(&self, path: &Path) -> Progress<anyhow::Result<FilesToOpen>> {
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
                RustProjectTemplate::Git { repository, branch } => {
                    generate_git(repository, branch)?
                }
                RustProjectTemplate::Local { path } => generate_local(path)?,
            }

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
            if !output.status.success() {
                return Err(From::from(Error::BuildingProjectFailed {
                    source: anyhow!("{}", String::from_utf8_lossy(&output.stderr)),
                }));
            }

            Ok(FilesToOpen {
                files: vec![path.join("main").join("src").join("main.rs")],
                directory: path.join("main"),
            })
        })
    }

    fn get_source(&self) -> anyhow::Result<RawSource> {
        stdfs::read_to_string("main/src/main.rs")
            .map(RawSource)
            .map_err(Into::into)
    }

    fn needs_compile(&self) -> bool {
        let target = if cfg!(windows) {
            "main/target/debug/main.exe"
        } else {
            "main/target/debug/main"
        };

        impfs::cmp_modified_time("main/src/main.rs", target)
            .map(|ord| ord == Ordering::Greater)
            .unwrap_or(true)
    }

    fn compile_command(&self) -> Command {
        Command::new("cargo").modify(|c| {
            c.arg("build").arg("--manifest-path").arg("main/Cargo.toml");
        })
    }

    fn run_command(&self) -> Command {
        Command::new("cargo").modify(|c| {
            c.arg("run")
                .arg("-q")
                .arg("--manifest-path")
                .arg("main/Cargo.toml");
        })
    }

    fn preprocess(&self, RawSource(source): &RawSource) -> anyhow::Result<Preprocessed> {
        Ok(Preprocessed(source.clone()))
    }

    fn minify(&self, Preprocessed(processed): &Preprocessed) -> anyhow::Result<Minified> {
        Ok(Minified(processed.clone()))
    }

    fn lint(&self, _minified: &Minified) -> Vec<String> {
        vec![]
    }
}

fn create_project_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|e| Error::CreateDestinationDirectoryFailed {
        source: e.into(),
        path: path.to_path_buf(),
    })
}

fn generate_git(repository: &str, branch: &str) -> anyhow::Result<()> {
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
    if !output.status.success() {
        return Err(From::from(Error::GeneratingProjectFailed {
            source: anyhow!("{}", String::from_utf8_lossy(&output.stderr)),
        }));
    }

    Ok(())
}

fn generate_local(path: &str) -> anyhow::Result<()> {
    dir::copy(path, "main", &CopyOptions::new())
        .map_err(|e| Error::GeneratingProjectFailed { source: e.into() })?;

    Ok(())
}

// fn get_lib_dir() -> PathBuf {
//     let mut home_dir = fs::get_home_path();
//     home_dir.push("procon-lib-rs");
//     home_dir
// }
