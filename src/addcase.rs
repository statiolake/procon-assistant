use common::open;
use imp::test_case::TestCaseFile;

use Result;

pub fn main() -> Result<()> {
    let tsf = TestCaseFile::new_with_next_unused_name()?;
    tsf.create()?;

    print_created!("{}, {}", tsf.input_file_name, tsf.output_file_name);

    open(&tsf.input_file_name)?;
    open(&tsf.output_file_name)?;

    Ok(())
}
