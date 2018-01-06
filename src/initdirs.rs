use std::path::PathBuf;
use std::fs;

pub fn main(args: Vec<String>) -> bool { 
    match args.len() {
        0 => {
            print_error!("please specify the contest name and number of problems.");
            return false;
        },
        1 => {
            print_error!("please specify the number of problems.");
            return false;
        },
        2 => {},
        _ => {
            print_error!("too many arguments for initdirs command.");
            return false;
        },
    }

    let contest_name = args[0].as_str();
    let numof_problems: u8 = match args[1].parse() {
        Ok(n) => n,
        Err(e) => {
            print_error!("failed to parse the number of problems: {}", e);
            return false;
        },
    };

    let mut dir_path = PathBuf::from(contest_name);
    for ch in (0..numof_problems).map(|x| (x + 'a' as u8) as char) {
        dir_path.push(ch.to_string());
        fs::create_dir_all(&dir_path).unwrap();
        dir_path.pop();
    }

    print_created!("directory tree");

    return true;
}
