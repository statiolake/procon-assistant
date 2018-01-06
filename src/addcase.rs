use std::process::Command;
use std::fs::File;
use std::path::Path;

fn ensure_create(name: &str) -> bool {
    let successful = File::create(name).is_ok();

    if !successful {
        print_error!("failed to create file {}.", name);
    }

    successful
}

fn spawn(name: &str) -> bool {
    if cfg!(windows) {
        let successful = Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(name)
            .output()
            .map(|x| println!("{:?}", x))
            .is_ok();

        if !successful {
            print_error!("failed to open case file.");
        }

        successful
    } else {
        unimplemented!();
    }
}

pub fn main() -> bool {
    let mut i = 1;
    while Path::new(&format!("in{}.txt", i)).exists() {
        i += 1;
    }

    let infile_name = format!("in{}.txt", i);
    let outfile_name = format!("out{}.txt", i);

    if Path::new(&outfile_name).exists() {
        print_error!("{} file exists while {} file doesn't exist.", outfile_name, infile_name);
        return false;
    }

    if !ensure_create(&infile_name) { return false; }
    if !ensure_create(&outfile_name) { return false; }

    if !spawn(&infile_name) { return false; }
    if !spawn(&outfile_name) { return false; }

    print_created!("{}, {}", infile_name, outfile_name);

    return true;
}
