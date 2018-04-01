pub mod aoj;

pub fn main(args: Vec<String>) -> bool {
    if args.is_empty() {
        print_error!("problem id not specified.");
        return false;
    }
    let arg = args.into_iter().next().unwrap();

    let (contest, id) = {
        let sp: Vec<_> = arg.split(':').collect();
        if sp.len() != 2 {
            print_error!("argument's format is not collect; please specify contest name and id separated by `:` (colon).");
            return false;
        }
        (sp[0], sp[1])
    };

    match contest {
        "aoj" => aoj::main(id),
        _ => {
            print_error!("the contest site {} is not available.", contest);
            false
        }
    }
}
