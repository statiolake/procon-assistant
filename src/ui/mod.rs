mod addcase;
mod clip;
mod compile;
mod delcase;
mod doc;
mod download;
mod fetch;
mod init;
mod initdirs;
mod login;
mod preprocess;
pub mod print_macros;
mod run;

use crate::imp::config::ConfigFile;
use crate::ExitStatus;
use crate::{eprintln_error, eprintln_info};
use anyhow::Result;
use clap::Clap;
use std::error;

#[derive(Clap)]
#[clap(version = "0.2", author = "statiolake")]
struct Options {
    #[clap(short, long)]
    quiet: bool,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "initdirs", aliases = &["id"])]
    InitDirs(initdirs::InitDirs),

    #[clap(name = "init", aliases = &["i"])]
    Init(init::Init),

    #[clap(name = "addcase", aliases = &["a", "ac"])]
    AddCase(addcase::AddCase),

    #[clap(name = "delcase", aliases = &["dc"])]
    DelCase(delcase::DelCase),

    #[clap(name = "doc", aliases = &["do"])]
    Doc(doc::Doc),

    #[clap(name = "preprocess", aliases = &["si", "pp"])]
    Preprocess(preprocess::Preprocess),

    #[clap(name = "clip", aliases = &["c"])]
    Clip(clip::Clip),

    #[clap(name = "fetch", aliases = &["f"])]
    Fetch(fetch::Fetch),

    #[clap(name = "download", aliases = &["d", "dl"])]
    Download(download::Download),

    #[clap(name = "run", aliases = &["r"])]
    Run(run::Run),

    #[clap(name = "compile", aliases = &["co"])]
    Compile(compile::Compile),

    #[clap(name = "login", aliases = &["l"])]
    Login(login::Login),
}

impl SubCommand {
    fn run(self, quiet: bool) -> Result<ExitStatus> {
        match self {
            SubCommand::InitDirs(cmd) => cmd.run(quiet),
            SubCommand::Init(cmd) => cmd.run(quiet),
            SubCommand::AddCase(cmd) => cmd.run(quiet),
            SubCommand::DelCase(cmd) => cmd.run(quiet),
            SubCommand::Preprocess(cmd) => cmd.run(quiet),
            SubCommand::Clip(cmd) => cmd.run(quiet),
            SubCommand::Fetch(cmd) => cmd.run(quiet),
            SubCommand::Download(cmd) => cmd.run(quiet),
            SubCommand::Run(cmd) => cmd.run(quiet),
            SubCommand::Compile(cmd) => cmd.run(quiet),
            SubCommand::Login(cmd) => cmd.run(quiet),
            SubCommand::Doc(cmd) => cmd.run(quiet),
        }
    }
}

pub fn main() -> ExitStatus {
    let _ = match ConfigFile::get_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln_error!("{}", e);
            print_causes(false, &*e);
            std::process::exit(1);
        }
    };

    let opts = Options::parse();
    match opts.subcommand.run(opts.quiet) {
        Ok(r) => r,
        Err(e) => {
            eprintln_error!("{}", e);
            print_causes(opts.quiet, &*e);
            ExitStatus::Failure
        }
    }
}

fn print_causes(quiet: bool, e: &dyn error::Error) {
    if quiet {
        return;
    }

    if let Some(cause) = e.source() {
        eprintln_info!("due to: {}", cause);
        print_causes(quiet, cause);
    }
}
