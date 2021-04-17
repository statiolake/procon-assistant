pub mod cpp;
pub mod python;
pub mod rust;

use self::cpp::Cpp;
use self::python::Python;
use self::rust::RustAtCoder2020;
use crate::imp::config::MinifyMode;
use crate::imp::progress::Progress;
use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use indexmap::indexmap;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::process::Command;

pub struct RawSource(pub String);
pub struct Preprocessed(pub String);

pub struct FilesToOpen {
    pub files: Vec<PathBuf>,
    pub directory: PathBuf,
}

pub trait Lang {
    fn check() -> Result<bool>
    where
        Self: Sized;

    fn new_boxed() -> Result<Box<dyn Lang>>
    where
        Self: Sized;

    fn lang_name() -> &'static str
    where
        Self: Sized;

    fn get_lang_name(&self) -> &'static str;

    fn init_async(&self) -> Progress<anyhow::Result<()>>;
    fn to_open(&self) -> Result<FilesToOpen>;
    fn open_docs(&self) -> Result<()>;
    fn needs_compile(&self) -> Result<bool>;
    fn needs_release_compile(&self) -> Result<bool>;
    fn get_source(&self) -> Result<RawSource>;
    fn compile_command(&self) -> Result<Vec<Command>>;
    fn release_compile_command(&self) -> Result<Vec<Command>>;
    fn run_command(&self) -> Result<Command>;
    fn release_run_command(&self) -> Result<Command>;
    fn preprocess(&self, source: &RawSource, minify: MinifyMode) -> Result<Preprocessed>;
    fn lint(&self, source: &RawSource) -> Result<Vec<String>>;
}

type CheckerType = fn() -> Result<bool>;
type CtorType = fn() -> Result<Box<dyn Lang>>;

lazy_static! {
    pub static ref LANGS_MAP: IndexMap<&'static str, (CheckerType, CtorType)> = indexmap! {
        Cpp::lang_name() => (Cpp::check as CheckerType, Cpp::new_boxed as CtorType),
        Python::lang_name() => (Python::check as CheckerType, Python::new_boxed as CtorType),
        RustAtCoder2020::lang_name() => (RustAtCoder2020::check as CheckerType, RustAtCoder2020::new_boxed as CtorType),
    };
    pub static ref FILETYPE_ALIAS: IndexMap<&'static str, &'static str> = indexmap! {
        Cpp::lang_name() => Cpp::lang_name(),
        RustAtCoder2020::lang_name() => RustAtCoder2020::lang_name(),
        Python::lang_name() => Python::lang_name(),
        "rust2020" => RustAtCoder2020::lang_name(),
        "rust" => RustAtCoder2020::lang_name(),
        "r" => RustAtCoder2020::lang_name(),
        "p" => Python::lang_name(),
    };
}

pub fn guess_lang() -> Result<Box<dyn Lang>> {
    for (name, (check, ctor)) in &*LANGS_MAP {
        if check().with_context(|| format!("failed to check for the language `{}`", name))? {
            return ctor().with_context(|| format!("failed to prepare the language `{}`", name));
        }
    }

    bail!("no language matched this project.")
}

pub fn get_from_alias(alias: &str) -> Result<Box<dyn Lang>> {
    let lang = FILETYPE_ALIAS
        .get(alias)
        .ok_or_else(|| anyhow!("unknown language: {}", alias))?;
    let (_, ctor) = LANGS_MAP
        .get(lang)
        .unwrap_or_else(|| panic!("internal error: unknown file type {}", lang));

    ctor().context("failed to prepare language service")
}
