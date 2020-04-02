use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create file `{file_name}`")]
    CreatingFailed {
        source: anyhow::Error,
        file_name: String,
    },

    #[error("failed to write to file `{file_name}`")]
    WritingFailed {
        source: anyhow::Error,
        file_name: String,
    },

    #[error("failed to check a metadata of file `{}`", .path.display())]
    CheckingMetadataFailed {
        source: anyhow::Error,
        path: PathBuf,
    },
}

/// Write the text into the specified file unless the file doesn't exist.  If
/// exists, return error.
pub fn safe_write(file_name: &str, text: &str) -> Result<()> {
    let mut f = File::create(file_name).map_err(|e| Error::CreatingFailed {
        source: e.into(),
        file_name: file_name.into(),
    })?;

    // if text is not empty, write the text into the file.
    if !text.is_empty() {
        f.write_all(text.as_bytes())
            .map_err(|e| Error::WritingFailed {
                source: e.into(),
                file_name: file_name.into(),
            })?;
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
        .map_err(|e| Error::CheckingMetadataFailed {
            source: e.into(),
            path: base.to_path_buf(),
        })?;

    let target = File::open(target)
        .and_then(|f| f.metadata())
        .and_then(|m| m.modified())
        .map_err(|e| Error::CheckingMetadataFailed {
            source: e.into(),
            path: target.to_path_buf(),
        })?;

    Ok(base.cmp(&target))
}
