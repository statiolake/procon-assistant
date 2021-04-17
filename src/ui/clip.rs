use crate::imp::config::CONFIG;
use crate::imp::langs::{Lang, Preprocessed};
use crate::imp::{clip, langs};
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_tagged, eprintln_warning};
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Copies the source file to clipboard with your library expanded")]
pub struct Clip;

impl Clip {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang =
            langs::guess_lang().context("failed to guess the language of the current project")?;
        if !quiet {
            eprintln_info!("guessed language: {}", lang.get_lang_name());
        }
        copy_to_clipboard(quiet, &*lang).context("failed to copy to the clipboard")?;

        Ok(ExitStatus::Success)
    }
}

pub fn copy_to_clipboard<L: Lang + ?Sized>(quiet: bool, lang: &L) -> Result<()> {
    eprintln_tagged!("Copying": "source file to clipboard");
    let source = lang.get_source().context("failed to load source code")?;
    let pped = lang
        .preprocess(&source, CONFIG.clip.minify)
        .context("failed to preprocess the source")?;

    let Preprocessed(mut pped) = pped;

    // Make sure the copied source ends with '\n'.
    pped.push('\n');

    // Set to the clipboard
    clip::set_clipboard(pped.clone());
    eprintln_tagged!("Finished": "copying");

    if !quiet {
        eprintln_info!("the copied string is as follows:");
        println!("{}", pped);
    }

    let lints = lang.lint(&source).context("failed to lint")?;
    if !lints.is_empty() {
        eprintln_warning!("linter found {} errors, is this OK?", lints.len());
        for lint in lints {
            eprintln_tagged!("Lint": "{}", lint);
        }
    }

    Ok(())
}
