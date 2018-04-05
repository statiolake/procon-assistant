use std::fs;
use std::num;
use std::path::PathBuf;

use Error;
use Result;

pub fn main(args: Vec<String>) -> Result<()> {
    let beginning_char = match args.len() {
        0 => Err(Error::new(
            "parsing argument",
            "please specify contest-name and the number of problems.",
            None,
        )),
        1 => Err(Error::new(
            "parsing argument",
            "please specify the number of problems.",
            None,
        )),
        2 => Ok('a'),
        3 if args[2].len() > 0 => Ok(args[2].chars().next().unwrap()),
        _ => Err(Error::new(
            "parsing argument",
            "too many arguments for initdirs command.",
            None,
        )),
    }?;

    let contest_name = args[0].as_str();
    let numof_problems: u8 = args[1].parse().map_err(|e: num::ParseIntError| {
        Error::new(
            "parsing the number of problems",
            "parse failed",
            Some(Box::new(e)),
        )
    })?;

    create_directories(contest_name, beginning_char, numof_problems);

    Ok(())
}

pub fn create_directories(contest_name: &str, beginning_char: char, numof_problems: u8) {
    let mut dir_path = PathBuf::from(contest_name);
    for ch in (0..numof_problems).map(|x| (x + beginning_char as u8) as char) {
        dir_path.push(ch.to_string());
        fs::create_dir_all(&dir_path).unwrap();
        dir_path.pop();
    }
    print_created!("directory tree");
}
