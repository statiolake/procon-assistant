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

        pub const LANG: Lang = Lang {
            file_type: "cpp",
            src_file_name: "main.cpp",
            compiler: "g++",
            flags: &[
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

        pub const PROCON_LIB_DIR: &str = "procon-lib";
        pub fn cmd_pre_modifier(cmd: &mut Command) -> Result<()> {
            cmd.arg(format!("-I{}", common::get_procon_lib_dir()?));
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
