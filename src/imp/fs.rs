use anyhow::{Context, Result};
use if_chain::if_chain;
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// Creates directories under `root`. If `override_if_cwd` is `true` and current
/// working directory name is equal to the root's name, the root directory is
/// not created (the current directory is assumed to be the root). Returns
/// actual root directory.
pub fn create_dirs(
    root: impl AsRef<str>,
    dirnames: &[impl AsRef<str>],
    override_if_cwd: bool,
) -> Result<PathBuf> {
    let root = root.as_ref();
    let cwd = env::current_dir().expect("critical error: failed to get current dir");
    let root = if_chain! {
        if override_if_cwd;
        if let Some(name) = cwd.file_name();
        if name == root;
        then {
            Path::new(".")
        } else {
            Path::new(root)
        }
    };

    fs::create_dir_all(root)
        .with_context(|| format!("failed to create a directory `{}`", root.display()))?;
    for dirname in dirnames {
        let path = root.join(dirname.as_ref());
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create a directory `{}`", path.display()))?;
    }

    Ok(root.to_path_buf())
}

/// Write the text into the specified file unless the file doesn't exist. If
/// exists, return error.
pub fn safe_write(file_name: &str, text: &str) -> Result<()> {
    let mut f =
        File::create(file_name).with_context(|| format!("failed to create `{}`", file_name))?;

    // if text is not empty, write the text into the file.
    if !text.is_empty() {
        f.write_all(text.as_bytes())
            .with_context(|| format!("failed to write into `{}`", file_name))?;
    }

    Ok(())
}

pub fn get_home_path() -> PathBuf {
    dirs::home_dir().expect("critical error: failed to get home_dir")
}

pub fn cmp_modified_time<P: AsRef<Path>, Q: AsRef<Path>>(base: P, target: Q) -> Result<Ordering> {
    let base = base.as_ref();
    let target = target.as_ref();

    let base = File::open(base)
        .and_then(|f| f.metadata())
        .and_then(|m| m.modified())
        .with_context(|| format!("checking metadata of base `{}` failed", base.display()))?;

    let target = File::open(target)
        .and_then(|f| f.metadata())
        .and_then(|m| m.modified())
        .with_context(|| format!("checking metadata of target `{}` failed", target.display()))?;

    Ok(base.cmp(&target))
}
