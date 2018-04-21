pub mod output_diff;

use colored_print::color::ConsoleColor::{self, *};

pub use self::output_diff::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct WrongAnswer {
    pub input: Vec<String>,
    pub expected_output: Vec<String>,
    pub actual_output: Vec<String>,
    pub difference: OutputDifference,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum JudgeResult {
    Passed,
    WrongAnswer(Option<WrongAnswer>),
    PresentationError,
    TimeLimitExceeded,
    RuntimeError(String), // reason
    CompilationError,
}

impl JudgeResult {
    pub fn color(&self) -> ConsoleColor {
        use self::JudgeResult::*;
        match *self {
            Passed => LightGreen,
            WrongAnswer(_) => Yellow,
            PresentationError => Yellow,
            TimeLimitExceeded => Yellow,
            RuntimeError(_) => Red,
            CompilationError => Yellow,
        }
    }

    pub fn long_name(&self) -> (ConsoleColor, &'static str, &'static str) {
        let color = self.color();
        use self::JudgeResult::*;
        let (verb, msg_to_be_colored) = match *self {
            Passed => ("", "Passed all sample case(s)"),
            WrongAnswer(_) => ("was ", "Wrong Answer"),
            PresentationError => ("was ", "Presentation Error"),
            TimeLimitExceeded => ("was ", "Time Limit Exceeded"),
            RuntimeError(_) => ("was ", "Runtime Error"),
            CompilationError => ("was ", "Compilation Error"),
        };

        (color, verb, msg_to_be_colored)
    }

    pub fn short_name(&self) -> (ConsoleColor, &'static str) {
        let color = self.color();
        use self::JudgeResult::*;
        let short_name = match *self {
            Passed => "PS ",
            WrongAnswer(_) => "WA ",
            PresentationError => "PE ",
            TimeLimitExceeded => "TLE",
            RuntimeError(_) => "RE ",
            CompilationError => "CE ",
        };

        (color, short_name)
    }
}
