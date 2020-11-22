use crate::imp::compile;
use crate::imp::compile::CompilerOutput;
use crate::imp::langs;
use crate::imp::langs::Lang;
use crate::ExitStatus;
use crate::{eprintln_info, eprintln_more, eprintln_tagged};
use anyhow::{Context, Result};

#[derive(clap::Clap)]
#[clap(about = "Compiles the current solution; the produced binary won't be tested automatically")]
pub struct Compile {
    #[clap(short, long, about = "Compiles in release mode")]
    release: bool,
    #[clap(
        short,
        long,
        about = "Recompiles even if the compiled binary seems to be up-to-date"
    )]
    force: bool,
}

impl Compile {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang =
            langs::guess_lang().context("failed to guess the language for the current project")?;

        let status = compile(quiet, self.release, &*lang, self.force)?;
        Ok(status)
    }
}

pub fn compile<L: Lang + ?Sized>(
    quiet: bool,
    release: bool,
    lang: &L,
    force: bool,
) -> Result<ExitStatus> {
    let needs_compile = if release {
        lang.needs_release_compile()
    } else {
        lang.needs_compile()
    }
    .context("failed to check compilation needs")?;

    if !force && !needs_compile {
        if !quiet {
            eprintln_info!("no need to compile");
        }
        return Ok(ExitStatus::Success);
    }

    do_compile(quiet, release, lang).context("failed to compile")
}

fn do_compile<L: Lang + ?Sized>(quiet: bool, release: bool, lang: &L) -> Result<ExitStatus> {
    eprintln_tagged!("Compiling": "project");
    let CompilerOutput {
        status,
        stdout,
        stderr,
    } = compile::compile(release, lang)?;
    print_compiler_output(quiet, "standard output", stdout);
    print_compiler_output(quiet, "standard error", stderr);

    Ok(status)
}

pub fn print_compiler_output(quiet: bool, kind: &str, output: Option<String>) {
    if quiet {
        return;
    }

    if let Some(output) = output {
        eprintln_info!("compiler {}:", kind);
        for line in output.trim().lines() {
            eprintln_more!("{}", line);
        }
    }
}
