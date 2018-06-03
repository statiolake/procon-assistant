use std::fs;
use std::path::PathBuf;

define_error!();
define_error_kind! {
    [InvalidNumberOfArgument; (n: usize, info: &'static str); format!(
        "invalid number of arguments for initdir command: {}\n{}", n, info
    )];
    [ParsingNumberOfProblemsFailed; (); format!(
        "failed to parse the number of problems."
    )];
}

pub fn main(args: Vec<String>) -> Result<()> {
    let beginning_char = match args.len() {
        0 => Err(Error::new(ErrorKind::InvalidNumberOfArgument(
            0,
            "please specify contest-name and the number of problems.",
        ))),
        1 => Err(Error::new(ErrorKind::InvalidNumberOfArgument(
            1,
            "please specify the number of problems.",
        ))),
        2 => Ok('a'),
        3 if args[2].len() > 0 => Ok(args[2].chars().next().unwrap()),
        n => Err(Error::new(ErrorKind::InvalidNumberOfArgument(
            n,
            "too many arguments",
        ))),
    }?;

    let contest_name = args[0].as_str();
    let numof_problems: u8 = args[1]
        .parse()
        .chain(ErrorKind::ParsingNumberOfProblemsFailed())?;

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
