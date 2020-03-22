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

use std::env;
use std::error;
use std::process;

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
    println!("        adds new sample case");
    println!("        creates inX.txt, outX.txt in current directory");
    println!("    delcase {{number}}");
    println!("        deletes specified existing sample case");
    println!("        removes inX.txt, outX.txt in current directory and");
    println!("        decrement the case number of succeeding sample cases");
    println!("        example:");
    println!("            assume there are three test cases now:");
    println!("                (in1.txt, out1.txt), (in2.txt, out2.txt), (in3.txt, out3.txt)");
    println!("            if you run `procon-assistant delcase 2`, then there are");
    println!("            two test cases:");
    println!("                (in1.txt, out1.txt), (in2.txt, out2.txt)");
    println!("            where (in2.txt, out2.txt) was previously (in3.txt, out3.txt)");
    println!("    preprocess");
    println!("        preprocess your source code and display it");
    println!("    clip");
    println!("        copy source file to clipboard (with library expanded)");
    println!("    fetch {{contest-site}}:{{problem-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site");
    println!("        examples:");
    println!("          - aoj:0123        problem of id 0123 in Aizu Online Judge");
    println!("          - atcoder:abc012a AtCoder Beginner Contest 012 Problem A");
    println!("          - atcoder:https://... the specified problem of AtCoder");
    println!("    download {{contest-site}}:{{contest-id}}");
    println!("        fetches sample cases of given problem-id in given contest-site");
    println!("        examples:");
    println!("          - atcoder:abc012  Atcoder Beginner Contest 012");
    println!("    run [test_case]");
    println!("        runs and tests current solution (main.cpp) with input inX.txt");
    println!("    compile");
    println!("        only compiles, no test is done");
    println!("    login {{contest-site}}");
    println!("        log in to the contest-site");
}

pub fn main() {
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
        "initdirs" | "id" => initdirs::main(quiet, args).map_err(anyhow::Error::from),
        "init" | "i" => init::main(quiet, args).map_err(anyhow::Error::from),
        "addcase" | "a" | "ac" => addcase::main(quiet).map_err(anyhow::Error::from),
        "delcase" | "dc" => delcase::main(quiet, args).map_err(anyhow::Error::from),
        "preprocess" | "si" | "pp" => preprocess::main(quiet).map_err(anyhow::Error::from),
        "clip" | "c" => clip::main(quiet).map_err(anyhow::Error::from),
        "fetch" | "f" => fetch::main(quiet, args).map_err(anyhow::Error::from),
        "download" | "d" | "dl" => download::main(quiet, args).map_err(anyhow::Error::from),
        "run" | "r" => run::main(quiet, args).map_err(anyhow::Error::from),
        "compile" | "co" => compile::main(quiet, args).map_err(anyhow::Error::from),
        "login" | "l" => login::main(quiet, args).map_err(anyhow::Error::from),
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
    if let Some(cause) = e.source() {
        print_info!(true, "due to: {}", cause);
        print_causes(cause);
    }
}
