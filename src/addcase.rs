use std::fs::File;
use std::io::Write;
use std::process::Command;

use common;

pub fn ensure_create(name: &str, text: &str) -> bool {
    if let Ok(mut f) = File::create(name) {
        if text != "" {
            match f.write_all(text.as_bytes()) {
                Ok(_) => (),
                Err(_) => {
                    print_error!("failed to write into case file");
                }
            }
        }
    } else {
        print_error!("failed to create file {}.", name);
        return false;
    }
    return true;
}

fn spawn(name: &str) {
    let successful = Command::new("open").arg(name).spawn().is_ok();

    if !successful {
        print_error!("failed to open case file. please manually open the file.");
    }
}

pub fn main() -> bool {
    let (infile_name, outfile_name) = match common::make_next_iofile_name() {
        Ok(r) => r,
        Err(_) => return false, // error message is displayed inside make_next_iofile_name()
    };

    if !ensure_create(&infile_name, "") {
        return false;
    }
    if !ensure_create(&outfile_name, "") {
        return false;
    }

    print_created!("{}, {}", infile_name, outfile_name);

    spawn(&infile_name);
    spawn(&outfile_name);

    return true;
}
