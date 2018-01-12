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

fn spawn(name: &str) {
    let successful = Command::new("open")
        .arg(name)
        .spawn()
        .is_ok();

    if !successful {
        print_error!("failed to open case file. please manually open the file.");
    }
}

pub fn main() -> bool {
    let mut i = 1;
    while Path::new(&::common::make_infile_name(i)).exists() {
        i += 1;
    }

    let infile_name = ::common::make_infile_name(i);
    let outfile_name = ::common::make_outfile_name(i);

    if Path::new(&outfile_name).exists() {
        print_error!("{} file exists while {} file doesn't exist.", outfile_name, infile_name);
        return false;
    }

    if !ensure_create(&infile_name) { return false; }
    if !ensure_create(&outfile_name) { return false; }

    print_created!("{}, {}", infile_name, outfile_name);

    spawn(&infile_name);
    spawn(&outfile_name);

    return true;
}
