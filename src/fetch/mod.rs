pub mod aoj;

pub fn main(arg: String) -> bool {
    let (contest, id) = {
        let sp: Vec<_> = arg.split(':').collect();
        (sp[0], sp[1])
    };

    match contest {
        "aoj" => aoj::main(id),
        _ => {
            print_err!("the contest site {} is not available.", contest);
            false
        }
    }
}
