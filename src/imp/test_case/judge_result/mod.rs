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

    pub fn long_name(&self) -> (Style, &'static str) {
        let style = self.style();
        let long_name = match *self {
            JudgeResult::Passed => "Passed",
            JudgeResult::WrongAnswer(_) => "Wrong Answer",
            JudgeResult::PresentationError => "Presentation Error",
            JudgeResult::TimeLimitExceeded => "Time Limit Exceeded",
            JudgeResult::RuntimeError(_) => "Runtime Error",
            JudgeResult::CompilationError => "Compilation Error",
        };

        (style, long_name)
    }

    pub fn short_name(&self) -> (Style, &'static str) {
        let style = self.style();
        let short_name = match *self {
            JudgeResult::Passed => "PS ",
            JudgeResult::WrongAnswer(_) => "WA ",
            JudgeResult::PresentationError => "PE ",
            JudgeResult::TimeLimitExceeded => "TLE",
            JudgeResult::RuntimeError(_) => "RE ",
            JudgeResult::CompilationError => "CE ",
        };

        (style, short_name)
    }
}
