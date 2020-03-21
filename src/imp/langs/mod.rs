use crate::imp::preprocess;
use crate::imp::preprocess::{Minified, Preprocessed, RawSource};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub mod cpp;
pub mod rust;

#[derive(Clone)]
pub struct Lang {
    pub lang: &'static str,
    pub src_file_name: &'static str,
    pub compiler: &'static str,
    pub lib_dir_getter: fn() -> PathBuf,
    pub compile_command_maker: fn(colorize: bool) -> Command,
    pub preprocessor: fn(quiet: bool, RawSource) -> preprocess::Result<Preprocessed>,
    pub minifier: fn(quiet: bool, Preprocessed) -> Minified,
    pub linter: fn(quiet: bool, &Minified) -> Vec<String>,
}

lazy_static! {
    pub static ref LANGS_MAP: HashMap<&'static str, Lang> = {
        let mut m = HashMap::new();
        m.insert(cpp::LANG.lang, cpp::LANG);
        m.insert(rust::LANG.lang, rust::LANG);
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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("failed to get the language for this project.")]
pub struct Error(ErrorKind);

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("there doesn't seem to have a supported file in the current dir.")]
    FileNotFound,
}

pub fn get_lang() -> Result<Lang> {
    for lang in LANGS {
        if Path::new(&lang.src_file_name).exists() {
            return Ok(lang.clone());
        }
    }
    Err(Error(ErrorKind::FileNotFound))
}
