pub mod cpp;
pub mod rust;

use self::cpp::Cpp;
use self::rust::Rust;
use indexmap::indexmap;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub struct RawSource(pub String);
pub struct Preprocessed(pub String);
pub struct Minified(pub String);

pub const PROGRESS_END_TOKEN: &str = "__PROGRESS__END__";

pub struct Progress<T> {
    pub handle: JoinHandle<T>,
    pub recver: mpsc::Receiver<String>,
}

impl<T: Send + 'static> Progress<T> {
    pub fn from_fn<F: (FnOnce(mpsc::Sender<String>) -> T) + Send + 'static>(f: F) -> Progress<T> {
        let (sender, recver) = mpsc::channel();
        let handle = thread::spawn(move || f(sender));
        Progress { handle, recver }
    }
}

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
    fn get_source(&self) -> anyhow::Result<RawSource>;
    fn compile_command(&self) -> Command;
    fn run_command(&self) -> Command;
    fn preprocess(&self, source: &RawSource) -> anyhow::Result<Preprocessed>;
    fn minify(&self, processed: &Preprocessed) -> anyhow::Result<Minified>;
    fn lint(&self, minified: &Minified) -> Vec<String>;
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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("failed to get the language for this project")]
pub struct Error(ErrorKind);

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("there doesn't seem to have a supported file in the current dir")]
    FileNotFound,
}

pub fn guess_language() -> Result<Box<dyn Language>> {
    LANGS_MAP
        .iter()
        .filter(|(_, (check, _))| check())
        .map(|(_, (_, ctor))| ctor())
        .next()
        .ok_or_else(|| Error(ErrorKind::FileNotFound))
}
