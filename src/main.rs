#![feature(box_syntax, str_escape, specialization)]
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
extern crate dirs;
extern crate encoding;
extern crate time;
#[macro_use]
extern crate lazy_static;

macro_rules! define_error {
    () => {
        pub type Result<T> = ::std::result::Result<T, Error>;

        #[derive(Debug)]
        pub struct Error {
            pub kind: ErrorKind,
            pub cause: Option<Box<::std::error::Error + Send>>,
        }

        pub trait ChainableToError<T> {
            fn chain(self, kind: ErrorKind) -> Result<T>;
        }

        impl Error {
            #[allow(dead_code)]
            pub fn new(kind: ErrorKind) -> Error {
                Error { kind, cause: None }
            }

            #[allow(dead_code)]
            pub fn with_cause(kind: ErrorKind, cause: Box<::std::error::Error + Send>) -> Error {
                Error {
                    kind,
                    cause: Some(cause),
                }
            }
        }

        impl ::std::error::Error for Error {
            fn cause(&self) -> Option<&::std::error::Error> {
                self.cause.as_ref().map(|e| &**e as &::std::error::Error)
            }
        }

        impl<T, E: 'static + ::std::error::Error + Send> ChainableToError<T>
            for ::std::result::Result<T, E>
        {
            fn chain(self, kind: ErrorKind) -> Result<T> {
                self.map_err(|e| Error::with_cause(kind, box e))
            }
        }
    };
}

macro_rules! define_error_kind {
    () => {
        #[derive(Debug)]
        pub enum ErrorKind {}
        impl ::std::fmt::Display for Error {
            fn fmt(&self, _: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                unreachable!();
            }
        }
    };
    ($([$id:ident; ($($cap:ident : $ty:ty),*); $ex:expr];)*) => {
        #[derive(Debug)]
        pub enum ErrorKind {
            $($id($($ty),*)),*
        }

        impl ::std::fmt::Display for Error {
            fn fmt(&self, b: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let message = match self.kind {
                    $(ErrorKind::$id($(ref $cap),*) => $ex),*
                };
                write!(b, "{}", message)
            }
        }
    };
}

#[macro_use]
mod tags;
mod addcase;

mod clip;
mod compile;
mod download;
mod fetch;
mod imp;
mod init;
mod initdirs;
mod login;
mod run;
mod solve_include;

use std::env;
use std::error;
use std::process;

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
    println!("    run [test_case]");
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
        "initdirs" | "id" => initdirs::main(args).map_err(box_err),
        "init" | "i" => init::main(args).map_err(box_err),
        "addcase" | "a" | "ac" => addcase::main().map_err(box_err),
        "solveinclude" | "si" => solve_include::main().map_err(box_err),
        "clip" | "c" => clip::main().map_err(box_err),
        "fetch" | "f" => fetch::main(args).map_err(box_err),
        "download" | "d" | "dl" => download::main(args).map_err(box_err),
        "run" | "r" => run::main(args).map_err(box_err),
        "compile" | "co" => compile::main().map_err(box_err),
        "login" | "l" => login::main(args).map_err(box_err),
        "--help" | "-h" => {
            help();
            return;
        }
        _ => {
            help();
            process::exit(1)
        }
    };

    if let Err(e) = result {
        print_error!("{}", e);
        print_causes(&*e);
        process::exit(1);
    }
}

fn print_causes(e: &dyn error::Error) {
    if let Some(cause) = e.cause() {
        print_info!(true, "due to: {}", cause);
        print_causes(cause);
    }
}

fn box_err<'a, E: error::Error + 'a>(e: E) -> Box<dyn error::Error + 'a> {
    box e as Box<dyn error::Error>
}
