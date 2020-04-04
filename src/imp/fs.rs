use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

/// Write the text into the specified file unless the file doesn't exist.  If
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
