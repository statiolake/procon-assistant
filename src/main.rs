#![feature(box_syntax, str_escape)]
#[macro_use]
extern crate colored_print;
extern crate clipboard;
extern crate isatty;
extern crate percent_encoding;
extern crate regex;
extern crate reqwest;
extern crate rpassword;
extern crate scraper;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate encoding;
extern crate time;

#[macro_use]
mod tags;
mod addcase;
mod clip;
mod common;
mod compile;
mod config;
mod download;
mod fetch;
mod imp;
mod init;
mod initdirs;
mod login;
mod run;
mod solve_include;

use std::env;
use std::fmt;
use std::process;

#[derive(Debug)]
pub struct Error {
    when: String,
    description: String,
    cause: Option<Box<dyn std::error::Error + Send>>,
}

impl Error {
    pub fn new<S, T>(when: S, description: T) -> Error
    where
        S: Into<String>,
        T: Into<String>,
    {
        Error {
            when: when.into(),
            description: description.into(),
            cause: None,
        }
    }

    pub fn with_cause<S, T>(
        when: S,
        description: T,
        cause: Box<dyn std::error::Error + Send>,
    ) -> Error
    where
        S: Into<String>,
        T: Into<String>,
    {
        Error {
            when: when.into(),
            description: description.into(),
            cause: Some(cause),
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

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.cause.as_ref().map(|x| &**x as &dyn std::error::Error)
    }
}

type Result<T> = std::result::Result<T, Error>;

fn help() {
    println!("Procon Assistant");
    println!("Usage: procon-assistant {{command}} [options]");
    println!("");
    println!("List of commands:");
    println!("    initdirs {{contest-name}} {{numof-problems}} [beginning-char]");
    println!("        initializes directories tree (name/{{a,...,a+num}})");
    println!("    init");
    println!("        initializes files in directory");
    println!("    clip");
    println!("        copy source file to clipboard (with library expanded)");
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
    println!("    login {{contest-site}}");
    println!("        log in to the contest-site.");
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        help();
        process::exit(1);
    }

    let cmd = args[1].clone();
    let args: Vec<_> = args.into_iter().skip(2).collect();
    let result = match cmd.as_str() {
        "initdirs" | "id" => initdirs::main(args),
        "init" | "i" => init::main(),
        "addcase" | "a" | "ac" => addcase::main(),
        "solveinclude" | "si" => solve_include::main(),
        "clip" | "c" => clip::main(),
        "fetch" | "f" => fetch::main(args),
        "download" | "d" | "dl" => download::main(args),
        "run" | "r" => run::main(args),
        "compile" | "co" => compile::main(),
        "login" | "l" => login::main(args),
        "--help" | "-h" => {
            help();
            return;
        }
        _ => {
            help();
            process::exit(1)
        }
    }.map_err(|e| Box::<dyn std::error::Error>::from(box e));

    if let Err(e) = result {
        print_error!("{}", e.description());
        if let Some(ref cause) = e.cause() {
            print_info!("due to {:?}", cause);
        }
        process::exit(1);
    }
}
