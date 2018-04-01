#[macro_use]
extern crate colored_print;
extern crate reqwest;
extern crate scraper;
extern crate time;

#[macro_use]
mod tags;
mod addcase;
mod common;
mod fetch;
mod init;
mod initdirs;
mod run;

use std::env;
use std::process;

fn help() {
    println!("Procon Assistant");
    println!("Usage: procon-assistant [command] [options]");
    println!("");
    println!("List of commands:");
    println!("    initdirs [name] [num] initializes directories tree (name/{{a,...,a+num}})");
    println!("    init           initializes files in directory");
    println!("    fetch [ID]     downloads test cases from webpages");
    println!("      [ID] is:");
    println!("      - aoj:xxxx        id xxxx of Aizu Online Judge");
    println!("      - atcoder:a?cXXXY Atcoder ? Contest XXX Problem Y");
    println!(
        "    addcase        adds new sample case. creates inX.txt, outX.txt in current directory."
    );
    println!("    run            runs and tests current solution (main.cpp) with input inX.txt.");
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
