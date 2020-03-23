#[macro_use]
pub(crate) mod tags;
mod addcase;
mod delcase;

mod clip;
mod compile;
mod download;
mod fetch;
mod init;
mod initdirs;
mod login;
mod preprocess;
mod run;

use clap::Clap;
use std::error;
use std::process;

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
    fn run(self, quiet: bool) -> anyhow::Result<()> {
        match self {
            SubCommand::InitDirs(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Init(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::AddCase(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::DelCase(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Preprocess(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Clip(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Fetch(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Download(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Run(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Compile(cmd) => cmd.run(quiet).map_err(Into::into),
            SubCommand::Login(cmd) => cmd.run(quiet).map_err(Into::into),
        }
    }
}

pub fn main() {
    let opts = Options::parse();
    let result = opts.subcommand.run(opts.quiet);
    if let Err(e) = result {
        print_error!("{}", e);
        print_causes(&*e);
        process::exit(1);
    }
}

fn print_causes(e: &dyn error::Error) {
    if let Some(cause) = e.source() {
        print_info!(true, "due to: {}", cause);
        print_causes(cause);
    }
}
