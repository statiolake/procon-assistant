use std::fs::File;
use std::io::Write;

use common;
use common::open;

use Error;
use Result;

pub fn ensure_create(name: &str, text: &str) -> Result<()> {
    let mut f = File::create(name)
        .map_err(|e| Error::with_cause(format!("creating {}", name), "failed", box e))?;

    if text != "" {
        f.write_all(text.as_bytes())
            .map_err(|e| Error::with_cause(format!("writing into {}", name), "failed", box e))?;
    }

    Ok(())
}

pub fn main() -> Result<()> {
    let (infile_name, outfile_name) = common::make_next_iofile_name().map_err(|_| {
        Error::new(
            "creating testcase file",
            "failed to generate testcase file's name.",
        )
    })?;

    ensure_create(&infile_name, "")?;
    ensure_create(&outfile_name, "")?;

    print_created!("{}, {}", infile_name, outfile_name);

    open(&infile_name)?;
    open(&outfile_name)?;

    Ok(())
}
