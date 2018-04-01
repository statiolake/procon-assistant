use std::path::Path;

pub fn make_next_iofile_name() -> Result<(String, String), ()> {
    let mut i = 1;
    while Path::new(&make_infile_name(i)).exists() {
        i += 1;
    }

    let infile_name = make_infile_name(i);
    let outfile_name = make_outfile_name(i);

    if Path::new(&outfile_name).exists() {
        print_error!(
            "{} file exists while {} file doesn't exist.",
            outfile_name,
            infile_name
        );
        return Err(());
    }

    Ok((infile_name, outfile_name))
}

pub fn make_infile_name(num: i32) -> String {
    format!("in{}.txt", num)
}

pub fn make_outfile_name(num: i32) -> String {
    format!("out{}.txt", num)
}
