use std::collections::HashMap;
use std::process::Command;

define_error!();
define_error_kind!{}

pub mod cpp;
pub mod rust;

pub struct Lang {
    pub file_type: &'static str,
    pub src_file_name: &'static str,
    pub compiler: &'static str,
    pub flags_setter: fn(&mut Command) -> Result<()>,
}

lazy_static! {
    pub static ref LANGS_MAP: HashMap<&'static str, Lang> = {
        let mut m = HashMap::new();
        m.insert("cpp", cpp::LANG);
        m.insert("rust", rust::LANG);
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
