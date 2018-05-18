use run;
use {Error, Result};

pub fn main() -> Result<()> {
    let res = run::compile()?;
    let successful = res.map_err(|err| Error::new("compiling", err))?;
    match successful {
        true => Ok(()),
        false => Err(Error::new("compiling", "build was not successful.")),
    }
}
