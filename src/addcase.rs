use common::open;
use imp::test_case::TestCaseFile;

use config::ConfigFile;

use Result;

pub fn main() -> Result<()> {
    let config: ConfigFile = ConfigFile::get_config()?;
    let tsf = TestCaseFile::new_with_next_unused_name()?;
    tsf.create()?;

    print_created!("{}, {}", tsf.input_file_name, tsf.output_file_name);

    open(&config.editor, &tsf.input_file_name)?;
    open(&config.editor, &tsf.output_file_name)?;

    Ok(())
}
