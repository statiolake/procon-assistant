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

pub const LANGS: &[Lang] = &[cpp::LANG, rust::LANG];
