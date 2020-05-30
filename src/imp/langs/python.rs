use super::Lang;
use super::{FilesToOpen, MinifyMode, Preprocessed, Progress, RawSource, Result};
use anyhow::Context;
use anyhow::{anyhow, bail};
use std::fs as stdfs;
use std::path::Path;
use std::process::Command;

pub struct Python;

impl Lang for Python {
    fn check() -> bool {
        Path::new("main.py").exists()
    }

    fn new_boxed() -> Box<dyn Lang> {
        Box::new(Python)
    }

    fn lang_name() -> &'static str {
        "python"
    }

    fn get_source(&self) -> Result<RawSource> {
        stdfs::read_to_string(Path::new("main.py"))
            .map(RawSource)
            .map_err(Into::into)
    }

    fn init_async(&self, path: &Path) -> Progress<anyhow::Result<()>> {
        let path_project = path.to_path_buf();
        Progress::from_fn(move |sender| {
            let _ = sender.send("creating main.py".into());
            let path_main = path_project.join("main.py");
            if !path_main.exists() {
                stdfs::write(path_main, "").context("failed to create main.py")?;
            }

            let _ = sender.send("generating Visual Studio Code settings".into());
            stdfs::create_dir_all(path_project.join(".vscode"))
                .context("failed to create .vscode dir")?;
            stdfs::write(
                path_project.join(".vscode").join("settings.json"),
                r#"{ "isProconProject": true }"#,
            )
            .context("failed to create Visual Studio Code settings")?;

            Ok(())
        })
    }

    fn to_open(&self, path: &Path) -> FilesToOpen {
        FilesToOpen {
            files: vec![path.join("main.py")],
            directory: path.to_path_buf(),
        }
    }

    fn open_docs(&self) -> Result<()> {
        bail!("TODO: Currently no documentation prepared for Python")
    }

    fn needs_compile(&self) -> bool {
        // Python does not need compilation
        false
    }

    fn compile_command(&self) -> Vec<Command> {
        // No need to compile
        vec![]
    }

    fn run_command(&self) -> Result<Command> {
        let py = which::which("python3")
            .or_else(|_| which::which("python"))
            .map_err(|_| anyhow!("failed to find python3 in your environment."))?;
        let mut cmd = Command::new(py);
        cmd.arg("main.py");
        Ok(cmd)
    }

    fn preprocess(&self, source: &RawSource, _minify: MinifyMode) -> Result<Preprocessed> {
        let RawSource(raw) = source;
        Ok(Preprocessed(raw.clone()))
    }

    fn lint(&self, _source: &RawSource) -> Result<Vec<String>> {
        Ok(vec![])
    }
}
