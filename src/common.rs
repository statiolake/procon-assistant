use std::path::Path;
use std::process::Command;

use Error;
use Result;

pub const COLORIZE: bool = false;

pub fn open(name: &str) -> Result<()> {
    Command::new("open")
        .arg(name)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            Error::with_cause(
                format!("opening {}", name),
                "failed to spawn open command.",
                box e,
            )
        })
}

pub fn make_next_iofile_name() -> Result<(String, String)> {
    let mut i = 1;
    while Path::new(&make_infile_name(i)).exists() {
        i += 1;
    }

    let infile_name = make_infile_name(i);
    let outfile_name = make_outfile_name(i);

    if Path::new(&outfile_name).exists() {
        return Err(Error::new(
            "generating next sample case file name",
            format!(
                "{} exists while {} doesn't exist.",
                outfile_name, infile_name
            ),
        ));
    }

    Ok((infile_name, outfile_name))
}

pub fn make_infile_name(num: i32) -> String {
    format!("in{}.txt", num)
}

pub fn make_outfile_name(num: i32) -> String {
    format!("out{}.txt", num)
}
