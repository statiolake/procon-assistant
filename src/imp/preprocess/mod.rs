use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod cpp;
pub mod rust;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("failed to preprocess the source code.")]
pub struct Error(ErrorKind);

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to load `{}`; file not found.", .path.display())]
    FileNotFound {
        #[source]
        source: anyhow::Error,
        path: PathBuf,
    },

    #[error("failed to canonicalize the path `{}`", .path.display())]
    CanonicalizationFailed {
        #[source]
        source: anyhow::Error,
        path: PathBuf,
    },
}

macro_rules! preprocessor_newtype {
    ($name:ident, $ty:ty, $clos:expr) => {
        pub struct $name($ty);
        impl $name {
            pub fn into_inner(self) -> $ty {
                let $name(inner) = self;
                inner
            }
            pub fn inner(&self) -> &$ty {
                let &$name(ref inner) = self;
                inner
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, b: &mut std::fmt::Formatter) -> std::fmt::Result {
                // to avoid clippy...
                let clos = $clos;
                write!(b, "{}", clos(self.inner()))
            }
        }
    };
}

preprocessor_newtype!(RawSource, String, |x| x);
preprocessor_newtype!(Preprocessed, Vec<String>, |x: &Vec<String>| x.join("\n"));
preprocessor_newtype!(Minified, String, |x| x);

pub fn read_source_file(file_path: &Path) -> Result<RawSource> {
    let mut src_content = String::new();
    File::open(file_path)
        .map_err(|e| {
            Error(ErrorKind::FileNotFound {
                source: e.into(),
                path: file_path.into(),
            })
        })?
        .read_to_string(&mut src_content)
        .unwrap_or_else(|_| {
            panic!(
                "critical error: failed to read `{}` from disk.",
                file_path.display()
            )
        });
    Ok(RawSource(src_content))
}
