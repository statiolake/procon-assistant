pub use self::output_diff::*;
use console::Style;

pub mod output_diff;

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
    pub fn style(&self) -> Style {
        use self::JudgeResult::*;
        match *self {
            Passed => Style::new().green().bold(),
            WrongAnswer(_) => Style::new().yellow(),
            PresentationError => Style::new().yellow(),
            TimeLimitExceeded => Style::new().yellow(),
            RuntimeError(_) => Style::new().red(),
            CompilationError => Style::new().yellow(),
        }
    }

    pub fn long_name(&self) -> (Style, &'static str, &'static str) {
        let style = self.style();
        use self::JudgeResult::*;
        let (verb, msg_to_be_colored) = match *self {
            Passed => ("", "Passed all sample case(s)"),
            WrongAnswer(_) => ("was ", "Wrong Answer"),
            PresentationError => ("was ", "Presentation Error"),
            TimeLimitExceeded => ("was ", "Time Limit Exceeded"),
            RuntimeError(_) => ("was ", "Runtime Error"),
            CompilationError => ("was ", "Compilation Error"),
        };

        (style, verb, msg_to_be_colored)
    }

    pub fn short_name(&self) -> (Style, &'static str) {
        let style = self.style();
        use self::JudgeResult::*;
        let short_name = match *self {
            Passed => "PS ",
            WrongAnswer(_) => "WA ",
            PresentationError => "PE ",
            TimeLimitExceeded => "TLE",
            RuntimeError(_) => "RE ",
            CompilationError => "CE ",
        };

        (style, short_name)
    }
}
