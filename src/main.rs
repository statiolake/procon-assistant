#[macro_use]
extern crate colored_print;
extern crate reqwest;
extern crate scraper;
extern crate time;

#[macro_use]
mod tags;
mod addcase;
mod common;
mod download;
mod fetch;
mod init;
mod initdirs;
mod run;

use std::env;
use std::fmt;
use std::process;

#[derive(Debug)]
pub struct Error {
    when: String,
    description: String,
    cause: Option<Box<std::error::Error>>,
}

impl Error {
    pub fn new<S, T>(when: S, description: T, cause: Option<Box<std::error::Error>>) -> Error
    where
        S: Into<String>,
        T: Into<String>,
    {
        Error {
            when: when.into(),
            description: description.into(),
            cause,
        }
    }

    pub fn display(&self) {
        print_error!("while {}: {}", self.when, self.description);
        if let Some(ref cause) = self.cause {
            print_info!("due to {:?}", cause);
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while {}: {}", self.when, self.description)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&std::error::Error> {
        self.cause.as_ref().map(|x| &**x)
    }
}

type Result<T> = std::result::Result<T, Option<Error>>;

fn help(with_flag: bool) -> Result<()> {
    println!("Procon Assistant");
    println!("Usage: procon-assistant {{command}} [options]");
    println!("");
    println!("List of commands:");
    println!("    initdirs {{contest-name}} {{numof-problems}} [beginning-char]");
    println!("        initializes directories tree (name/{{a,...,a+num}})");
    println!("    init");
    println!("        initializes files in directory");
    println!("    fetch {{contest-site}}:{{problem-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site.");
    println!("        examples:");
    println!("          - aoj:0123        problem of id 0123 in Aizu Online Judge");
    println!("          - atcoder:abc012a Atcoder Beginner Contest 012 Problem A");
    println!("    download {{contest-site}}:{{contest-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site.");
    println!("        examples:");
    println!("          - atcoder:abc012  Atcoder Beginner Contest 012");
    println!("    addcase");
    println!("        adds new sample case.");
    println!("        creates inX.txt, outX.txt in current directory.");
    println!("    run [testcase]");
    println!("        runs and tests current solution (main.cpp) with input inX.txt.");

    if with_flag {
        Ok(())
    } else {
        Err(None)
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        match help(false) {
            _ => (),
        }
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "initdirs" | "id" => initdirs::main(args.into_iter().skip(2).collect()),
        "init" | "i" => init::main(),
        "addcase" | "a" | "ac" => addcase::main(),
        "fetch" | "f" => fetch::main(args.into_iter().skip(2).collect()),
        "download" | "d" | "dl" => download::main(args.into_iter().skip(2).collect()),
        "run" | "r" => run::main(args.into_iter().skip(2).collect()),
        "--help" | "-h" => help(true),
        _ => help(false),
    };

    if let Err(e) = result {
        if let Some(e) = e {
            e.display();
        }
        process::exit(1);
    }
}
