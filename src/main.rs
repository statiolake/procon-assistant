pub mod imp;
pub mod ui;

use std::process::ExitCode;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Failure,
}

fn main() -> ExitCode {
    match ui::main() {
        ExitStatus::Success => ExitCode::SUCCESS,
        ExitStatus::Failure => ExitCode::FAILURE,
    }
}
