use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use imp::preprocess;
use imp::preprocess::{Minified, Preprocessed, RawSource};

pub mod cpp;
pub mod rust;

#[derive(Clone)]
pub struct Lang {
    pub file_type: &'static str,
    pub src_file_name: &'static str,
    pub compiler: &'static str,
    pub lib_dir_getter: fn() -> PathBuf,
    pub compile_command_maker: fn() -> Command,
    pub preprocessor: fn(RawSource) -> preprocess::Result<Preprocessed>,
    pub minifier: fn(Preprocessed) -> Minified,
}

lazy_static! {
    pub static ref LANGS_MAP: HashMap<&'static str, Lang> = {
        let mut m = HashMap::new();
        m.insert(cpp::LANG.file_type, cpp::LANG);
        m.insert(rust::LANG.file_type, rust::LANG);
        m
    };
    pub static ref FILETYPE_ALIAS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("cpp", "cpp");
        m.insert("rust", "rust");
        m.insert("r", "rust");
        m
    };
}

pub const LANGS: &[Lang] = &[cpp::LANG, rust::LANG];

define_error!();
define_error_kind! {
    [FileNotFound; (); "there seems not to have supported file in current dir."];
}

pub fn get_lang() -> Result<Lang> {
    for lang in LANGS {
        if Path::new(&lang.src_file_name).exists() {
            return Ok(lang.clone());
        }
    }
    return Err(Error::new(ErrorKind::FileNotFound()));
}