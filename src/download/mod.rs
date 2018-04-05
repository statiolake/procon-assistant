mod print_msg;

pub mod atcoder;

pub fn main(args: Vec<String>) -> bool {
    if args.is_empty() {
        print_error!("contest-site and contest-id are not specified.");
        return false;
    }
    let arg = args.into_iter().next().unwrap();

    let (site, id) = {
        let sp: Vec<_> = arg.split(':').collect();
        if sp.len() != 2 {
            print_error!("argument's format is not collect; please specify contest name and id separated by `:` (colon).");
            return false;
        }
        (sp[0], sp[1])
    };

    match site {
        "atcoder" => atcoder::main(id),
        _ => {
            print_error!("the contest site {} is not available.", site);
            false
        }
    }
}
