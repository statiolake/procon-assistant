use serde_json;

use std::env;
use std::fs::File;

use {Error, Result};

pub const TIMEOUT_MILLISECOND: i64 = 3000;
pub const HEADER_FILE_EXTENSIONS: &[&str] = &["h", "hpp"];

#[derive(Deserialize)]
pub struct ConfigFile {
    pub editor: String,
    pub auto_open: bool,
    pub open_directory_instead_of_specific_file: bool,
}

impl ConfigFile {
    pub fn get_config() -> Result<ConfigFile> {
        let config_path = env::current_exe()
            .map_err(|e| {
                Error::with_cause(
                    "getting config",
                    "failed to get directory of executable.",
                    box e,
                )
            })?
            .with_file_name("config.json");
        let f = File::open(&config_path).map_err(|e| {
            Error::with_cause("getting config", "failed to open `config.json`", box e)
        })?;
        serde_json::from_reader(f).map_err(|e| {
            Error::with_cause("getting config", "failed to parse `config.json`", box e)
        })
    }
}

pub mod src_support {
    use std::process::Command;
    use Result;

    pub struct Lang {
        pub file_type: &'static str,
        pub src_file_name: &'static str,
        pub compiler: &'static str,
        pub flags: &'static [&'static str],
        pub cmd_pre_modifier: Option<fn(&mut Command) -> Result<()>>,
    }

    pub const LANGS: &[Lang] = &[cpp::LANG, rust::LANG];

    pub mod cpp {
        use super::Lang;
        use common;
        use std::process::Command;
        use Result;

        #[cfg(unix)]
        pub const LANG: Lang = Lang {
            file_type: "cpp",
            src_file_name: "main.cpp",
            compiler: "g++",
            flags: &[
                "-g",
                "-std=c++14",
                "-Wall",
                "-Wextra",
                "-Wno-old-style-cast",
                "-DPA_DEBUG",
                "-omain",
                "main.cpp",
            ],
            cmd_pre_modifier: Some(cmd_pre_modifier),
        };
        #[cfg(windows)]
        pub const LANG: Lang = Lang {
            file_type: "cpp",
            src_file_name: "main.cpp",
            compiler: "cmd",
            flags: &[],
            cmd_pre_modifier: Some(cmd_pre_modifier),
        };

        pub const PROCON_LIB_DIR: &str = "procon-lib";
        #[cfg(unix)]
        pub fn cmd_pre_modifier(cmd: &mut Command) -> Result<()> {
            cmd.arg(format!("-I{}", common::get_procon_lib_dir()?.display()).escape_default());
            Ok(())
        }

        #[cfg(windows)]
        pub fn cmd_pre_modifier(cmd: &mut Command) -> Result<()> {
            cmd.args(&[
                "/c",
                "vsprompt.bat",
                "cl",
                "/EHsc",
                "/Zi",
                "/source-charset:utf-8",
                "/DPA_DEBUG",
            ]);
            cmd.arg(format!("/I{}", common::get_procon_lib_dir()?.display()).escape_default());
            cmd.arg("main.cpp");
            Ok(())
        }
    }

    pub mod rust {
        use super::Lang;
        pub const LANG: Lang = Lang {
            file_type: "rust",
            src_file_name: "main.rs",
            compiler: "rustc",
            flags: &["main.rs"],
            cmd_pre_modifier: None,
        };
    }
}
