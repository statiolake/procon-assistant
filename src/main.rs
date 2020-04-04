pub mod imp;
pub mod ui;

use std::process;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Failure,
}

fn main() {
    if ui::main() == ExitStatus::Failure {
        process::exit(1);
    }
}
