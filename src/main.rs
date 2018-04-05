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
use std::process;

fn help() {
    println!("Procon Assistant");
    println!("Usage: procon-assistant {{command}} [options]");
    println!("");
    println!("List of commands:");
    println!("    initdirs {{name}} {{num}} [beginning-char]");
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
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        help();
        process::exit(1);
    }

    let successful = match args[1].as_str() {
        "initdirs" | "id" => initdirs::main(args.into_iter().skip(2).collect()),
        "init" | "i" => init::main(),
        "addcase" | "a" | "ac" => addcase::main(),
        "fetch" | "f" => fetch::main(args.into_iter().skip(2).collect()),
        "download" | "d" | "dl" => download::main(args.into_iter().skip(2).collect()),
        "run" | "r" => run::main(args.into_iter().skip(2).collect()),
        "--help" | "-h" => {
            help();
            true
        }
        _ => {
            help();
            false
        }
    };

    if !successful {
        process::exit(1);
    }
}
