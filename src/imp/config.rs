use serde_json;

use std::env;
use std::fs::File;
use tags::SPACER;

pub const TIMEOUT_MILLISECOND: i64 = 3000;
pub const HEADER_FILE_EXTENSIONS: &[&str] = &["h", "hpp"];

define_error!();
define_error_kind! {
    [ConfigFileMissing; (); format!(concat!(
        "`config.json' is missing.\n",
        "{}please check that file is placed at the same directory where this binary is placed."
    ), SPACER)];
    [ErrorInConfigFile; (); format!(concat!(
        "failed to parse config.json.\n",
        "{}maybe that file has syntax error, unknown options, or mismatched types."
    ), SPACER)];
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub editor: String,
    pub init_auto_open: bool,
    pub init_open_directory_instead_of_specific_file: bool,
    pub init_default_file_type: String,
    // for terminal editor like vim
    pub addcase_wait_editor_finish: bool,
    pub addcase_give_argument_once: bool,
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = env::current_exe().unwrap().with_file_name("config.json");
        File::open(&config_path)
            .chain(ErrorKind::ConfigFileMissing())
            .and_then(|f| serde_json::from_reader(f).chain(ErrorKind::ErrorInConfigFile()))
    }
}

pub mod src_support {
    use std::process::Command;

    use super::Result;

    pub struct Lang {
        pub file_type: &'static str,
        pub src_file_name: &'static str,
        pub compiler: &'static str,
        pub flags_setter: fn(&mut Command) -> Result<()>,
    }

    pub const LANGS: &[Lang] = &[cpp::LANG, rust::LANG];

    pub mod cpp {
        use std::process::Command;

        use super::Lang;
        use super::Result;
        use imp::common;

        pub const PROCON_LIB_DIR: &str = "procon-lib";

        // #[cfg(not(windows))]
        pub const LANG: Lang = Lang {
            file_type: "cpp",
            src_file_name: "main.cpp",
            compiler: "g++",
            flags_setter: flags_setter,
        };

        // #[cfg(not(windows))]
        pub fn flags_setter(cmd: &mut Command) -> Result<()> {
            cmd.arg(format!("-I{}", common::get_procon_lib_dir().display()).escape_default());
            cmd.args(&[
                "-g",
                "-std=c++14",
                "-Wall",
                "-Wextra",
                "-Wno-old-style-cast",
                "-DPA_DEBUG",
                "-omain",
                "main.cpp",
            ]);
            if cfg!(unix) {
                cmd.arg("-fdiagnostics-color=always");
            }
            Ok(())
        }

        /*
        #[cfg(windows)]
        pub const LANG: Lang = Lang {
            file_type: "cpp",
            src_file_name: "main.cpp",
            compiler: "cmd",
            flags_setter: flags_setter,
        };
        
        #[cfg(windows)]
        pub fn flags_setter(cmd: &mut Command) -> Result<()> {
            cmd.args(&[
                "/c",
                "vsprompt.bat",
                "cl",
                "/EHsc",
                "/Zi",
                "/source-charset:utf-8",
                "/DPA_DEBUG",
            ]);
            cmd.arg(format!("/I{}", common::get_procon_lib_dir().display()).escape_default());
            cmd.arg("main.cpp");
            Ok(())
        }
        */
    }

    pub mod rust {
        use super::Lang;
        use super::Result;

        use std::process::Command;

        pub const LANG: Lang = Lang {
            file_type: "rust",
            src_file_name: "main.rs",
            compiler: "rustc",
            flags_setter: flags_setter,
        };

        pub fn flags_setter(cmd: &mut Command) -> Result<()> {
            cmd.args(&["main.rs"]);
            Ok(())
        }
    }
}
