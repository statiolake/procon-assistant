#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OutputDifference {
    SizeDiffers,
    Different(Vec<usize>),
    NotDifferent,
}

impl OutputDifference {
    pub fn message(&self) -> String {
        match *self {
            OutputDifference::SizeDiffers => "the number of output lines is different".to_string(),
            OutputDifference::NotDifferent => unreachable!(), // this should be treated as Presentation Error.
            OutputDifference::Different(ref different_lines) => {
                let message = different_lines
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(&", ".to_string());
                format!("line {} differs", message)
            }
        }
    }
}

pub fn enumerate_different_lines(expected: &[String], actual: &[String]) -> OutputDifference {
    if expected.len() != actual.len() {
        return OutputDifference::SizeDiffers;
    }

    let mut different_lines = vec![];
    for i in 0..expected.len() {
        if expected[i] != actual[i] {
            different_lines.push(i + 1);
        }
    }

    if different_lines.is_empty() {
        // this is not wrong answer, but maybe presentation error;
        OutputDifference::NotDifferent
    } else {
        OutputDifference::Different(different_lines)
    }
}
