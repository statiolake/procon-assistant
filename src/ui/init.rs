use crate::eprintln_progress;
use crate::imp::common;
use crate::imp::config::ConfigFile;
use crate::imp::langs;
use crate::ExitStatus;
use std::path::Path;

#[derive(clap::Clap)]
#[clap(about = "Generates files in a directory")]
pub struct Init {
    #[clap(
        default_value = ".",
        help = "The name of directory;  if `.`, files will be generated in the current directory"
    )]
    dirname: String,

    #[clap(short, long, help = "The language to init")]
    lang: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get config")]
    GettingConfigFailed { source: anyhow::Error },

    #[error("unknown file type: {lang}")]
    UnknownFileType { lang: String },

    #[error("failed to open editor")]
    OpeningEditorFailed { source: anyhow::Error },

    #[error("init thread panicked")]
    WaitingForInitFinishFailed,

    #[error("init failed")]
    InitFailed { source: anyhow::Error },
}

impl Init {
    pub fn run(self, _quiet: bool) -> Result<ExitStatus> {
        let config: ConfigFile = ConfigFile::get_config()
            .map_err(|e| Error::GettingConfigFailed { source: e.into() })?;

        let specified_lang = self
            .lang
            .unwrap_or_else(|| config.init_default_lang.clone());

        let lang =
            langs::FILETYPE_ALIAS
                .get(&*specified_lang)
                .ok_or_else(|| Error::UnknownFileType {
                    lang: specified_lang,
                })?;

        let (_, ctor) = langs::LANGS_MAP
            .get(lang)
            .unwrap_or_else(|| panic!("internal error: unknown file type {}", lang));
        let lang = ctor();
        let path_project = Path::new(&self.dirname);

        // initialize the project asynchronously and get progress
        let progress = lang.init_async(path_project);
        while let Ok(msg) = progress.recver.recv() {
            eprintln_progress!("{}", msg);
        }

        let to_open = progress
            .handle
            .join()
            .map_err(|_| Error::WaitingForInitFinishFailed)?
            .map_err(|source| Error::InitFailed { source })?;

        let to_open = if config.init_open_directory_instead_of_specific_file {
            vec![to_open.directory]
        } else {
            to_open.files
        };

        if config.init_auto_open {
            common::open_general(&config, &to_open)
                .map_err(|e| Error::OpeningEditorFailed { source: e.into() })?;
        }

        Ok(ExitStatus::Success)
    }
}
