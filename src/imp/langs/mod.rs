pub mod cpp;
pub mod rust;

use self::cpp::Cpp;
use self::rust::Rust;
use crate::imp::config::MinifyMode;
use crate::imp::progress::Progress;
use anyhow::anyhow;
use anyhow::Result;
use indexmap::indexmap;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RawSource(pub String);
pub struct Preprocessed(pub String);

pub struct FilesToOpen {
    pub files: Vec<PathBuf>,
    pub directory: PathBuf,
}

pub trait Language {
    fn check() -> bool
    where
        Self: Sized;

    fn new_boxed() -> Box<dyn Language>
    where
        Self: Sized;

    fn language_name() -> &'static str
    where
        Self: Sized;

    fn init_async(&self, path: &Path) -> Progress<anyhow::Result<FilesToOpen>>;
    fn needs_compile(&self) -> bool;
    fn get_source(&self) -> Result<RawSource>;
    fn compile_command(&self) -> Vec<Command>;
    fn run_command(&self) -> Command;
    fn preprocess(&self, source: &RawSource, minify: MinifyMode) -> Result<Preprocessed>;
    fn lint(&self, source: &RawSource) -> Result<Vec<String>>;
}

type CheckerType = fn() -> bool;
type CtorType = fn() -> Box<dyn Language>;

lazy_static! {
    pub static ref LANGS_MAP: IndexMap<&'static str, (CheckerType, CtorType)> = indexmap! {
        Cpp::language_name() => (Cpp::check as CheckerType, Cpp::new_boxed as CtorType),
        Rust::language_name() => (Rust::check as CheckerType, Rust::new_boxed as CtorType),
    };
    pub static ref FILETYPE_ALIAS: IndexMap<&'static str, &'static str> = indexmap! {
        Cpp::language_name() => Cpp::language_name(),
        Rust::language_name() => Rust::language_name(),
        "r" => Rust::language_name(),
    };
}

pub fn guess_language() -> Result<Box<dyn Language>> {
    LANGS_MAP
        .iter()
        .filter(|(_, (check, _))| check())
        .map(|(_, (_, ctor))| ctor())
        .next()
        .ok_or_else(|| anyhow!("no language is match"))
}
