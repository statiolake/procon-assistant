use crate::imp::clip;
use crate::imp::langs;
use crate::imp::langs::{Language, Minified, Preprocessed};
use crate::ui::CONFIG;
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_tagged, eprintln_warning};
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Copies the source file to clipboard with your library expanded")]
pub struct Clip;

impl Clip {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_language().context("failed to guess the current language")?;
        copy_to_clipboard(quiet, &*lang).context("failed to copy to the clipboard")?;

        Ok(ExitStatus::Success)
    }
}

pub fn copy_to_clipboard<L: Language + ?Sized>(quiet: bool, lang: &L) -> Result<()> {
    eprintln_tagged!("Copying": "source file to clipboard");
    let source = lang.get_source().context("failed to load source code")?;
    let preprocessed = lang
        .preprocess(&source)
        .context("failed to preprocess the source")?;
    let minified = if CONFIG.clip.minify {
        lang.minify(&preprocessed)
            .context("failed to minify the source")?
    } else {
        let Preprocessed(p) = preprocessed;
        Minified(p)
    };

    let lints = lang.lint(&minified);
    let Minified(minified) = minified;
    let minified = minified + "\n";
    clip::set_clipboard(minified.clone());
    eprintln_tagged!("Finished": "copying");

    if !quiet {
        eprintln_info!("the copied string is as follows:");
        println!("{}", minified);
    }

    if !lints.is_empty() {
        eprintln_warning!("linter found {} errors, is this OK?", lints.len());

        for lint in lints {
            eprintln_tagged!("Lint": "{}", lint);
        }
    }

    Ok(())
}
