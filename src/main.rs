macro_rules! define_error {
    () => {
        pub type Result<T> = std::result::Result<T, Error>;

        #[derive(Debug)]
        pub struct Error {
            pub kind: ErrorKind,
            pub cause: Option<Box<std::error::Error + Send>>,
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
            pub fn with_cause(kind: ErrorKind, cause: Box<std::error::Error + Send>) -> Error {
                Error {
                    kind,
                    cause: Some(cause),
                }
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.cause.as_ref().map(|e| &**e as &std::error::Error)
            }
        }

        impl crate::ErrorWithSilent for Error {
            fn is_silent(&self) -> bool {
                match self.kind {
                    ErrorKind::SilentError => true,
                    _ => false,
                }
            }

            fn upcast(&self) -> &(dyn std::error::Error + Send) {
                self
            }
        }

        impl<T, E: 'static + std::error::Error + Send> ChainableToError<T>
            for std::result::Result<T, E>
        {
            fn chain(self, kind: ErrorKind) -> Result<T> {
                self.map_err(|e| Error::with_cause(kind, Box::new(e)))
            }
        }
    };
}

macro_rules! define_error_kind {
    () => {
        #[derive(Debug)]
        pub enum ErrorKind {
            #[allow(dead_code)] SilentError,
        }
        impl std::fmt::Display for ErrorKind {
            fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
                unreachable!("SilentError must not be formatted.");
            }
        }
    };
    ($([$id:ident; ($($cap:ident : $ty:ty),*); $ex:expr];)*) => {
        #[derive(Debug)]
        pub enum ErrorKind {
            /// quietly stops the application with error state. this is used in
            /// `run` command, when some test case fails. no other error
            /// messages are needed, but run command should return error value
            /// when some test fails, so that other utilities can get the test
            /// result.
            #[allow(dead_code)] SilentError,
            $($id($($ty),*)),*
        }

        impl std::fmt::Display for Error {
            fn fmt(&self, b: &mut std::fmt::Formatter) -> std::fmt::Result {
                let message = match self.kind {
                    // do nothing when SilentError
                    ErrorKind::SilentError => unreachable!("SilentError must not be formatted."),
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
mod delcase;

mod clip;
mod compile;
mod download;
mod fetch;
mod imp;
mod init;
mod initdirs;
mod login;
mod preprocess;
mod run;

use std::env;
use std::error;
use std::process;

trait ErrorWithSilent: error::Error + Send {
    fn is_silent(&self) -> bool;
    fn upcast(&self) -> &(dyn error::Error + Send);
}

fn help() {
    println!("Procon Assistant");
    println!("Usage: procon-assistant {{command}} [options]");
    println!();
    println!("List of commands:");
    println!("    initdirs {{contest-name}} {{numof-problems}} [beginning-char]");
    println!("        initializes directories tree (name/{{a,...,a+num}})");
    println!("    init");
    println!("        initializes files in directory");
    println!("    addcase");
    println!("        adds new sample case.");
    println!("        creates inX.txt, outX.txt in current directory.");
    println!("    delcase {{number}}");
    println!("        deletes specified existing sample case.");
    println!("        removes inX.txt, outX.txt in current directory and");
    println!("        decrement the case number of succeeding sample cases.");
    println!("        example:");
    println!("            assume there are three test cases now:");
    println!("                (in1.txt, out1.txt), (in2.txt, out2.txt), (in3.txt, out3.txt)");
    println!("            if you run `procon-assistant delcase 2`, then there are");
    println!("            two test cases:");
    println!("                (in1.txt, out1.txt), (in2.txt, out2.txt)");
    println!("            where (in2.txt, out2.txt) was previously (in3.txt, out3.txt).");
    println!("    preprocess");
    println!("        preprocess your source code and display it.");
    println!("    clip");
    println!("        copy source file to clipboard (with library expanded)");
    println!("    fetch {{contest-site}}:{{problem-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site.");
    println!("        examples:");
    println!("          - aoj:0123        problem of id 0123 in Aizu Online Judge");
    println!("          - atcoder:abc012a AtCoder Beginner Contest 012 Problem A");
    println!("          - atcoder:https://... the specified problem of AtCoder");
    println!("    download {{contest-site}}:{{contest-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site.");
    println!("        examples:");
    println!("          - atcoder:abc012  Atcoder Beginner Contest 012");
    println!("    run [test_case]");
    println!("        runs and tests current solution (main.cpp) with input inX.txt.");
    println!("    compile");
    println!("        only compiles, no test is done.");
    println!("    login {{contest-site}}");
    println!("        log in to the contest-site.");
}

fn main() {
    let quiet = env::args().any(|x| x == "--quiet" || x == "-q");
    let args: Vec<_> = env::args()
        .filter(|x| x != "--quiet" && x != "-q")
        .collect();

    if args.len() < 2 {
        help();
        process::exit(1);
    }

    let cmd = args[1].clone();
    let args: Vec<_> = args.into_iter().skip(2).collect();
    let result = match cmd.as_str() {
        "initdirs" | "id" => initdirs::main(quiet, args).map_err(box_err),
        "init" | "i" => init::main(quiet, args).map_err(box_err),
        "addcase" | "a" | "ac" => addcase::main(quiet).map_err(box_err),
        "delcase" | "dc" => delcase::main(quiet, args).map_err(box_err),
        "preprocess" | "si" | "pp" => preprocess::main(quiet).map_err(box_err),
        "clip" | "c" => clip::main(quiet).map_err(box_err),
        "fetch" | "f" => fetch::main(quiet, args).map_err(box_err),
        "download" | "d" | "dl" => download::main(quiet, args).map_err(box_err),
        "run" | "r" => run::main(quiet, args).map_err(box_err),
        "compile" | "co" => compile::main(quiet, args).map_err(box_err),
        "login" | "l" => login::main(quiet, args).map_err(box_err),
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
        if !e.is_silent() {
            print_error!("{}", e);
            print_causes(e.upcast());
        }
        process::exit(1);
    }
}

fn print_causes(e: &dyn error::Error) {
    if let Some(cause) = e.source() {
        print_info!(true, "due to: {}", cause);
        print_causes(cause);
    }
}

fn box_err<'a, E: ErrorWithSilent + 'a>(e: E) -> Box<dyn ErrorWithSilent + 'a> {
    Box::new(e)
}
