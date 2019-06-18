use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod cpp;
pub mod rust;

define_error!();
define_error_kind! {
    [FileNotFound; (file_name: String); format!("failed to load '{}'; file not found.", file_name)];
    [CanonicalizationFailed; (path: PathBuf); format!("failed to canonicalize '{}'", path.display())];
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
        .chain(ErrorKind::FileNotFound(file_path.display().to_string()))?
        .read_to_string(&mut src_content)
        .unwrap_or_else(|_| {
            panic!(
                "critical error: failed to read `{}' from disk.",
                file_path.display()
            )
        });
    Ok(RawSource(src_content))
}
