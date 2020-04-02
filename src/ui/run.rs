use crate::imp::langs;
use crate::imp::langs::Language;
use crate::imp::test_case;
use crate::imp::test_case::{
    Context, JudgeResult, RuntimeErrorKind, Span, TestCase, TestResult, WrongAnswer,
    WrongAnswerKind,
};
use crate::ui::clip;
use crate::ui::compile;
use crate::ui::print_macros::TAG_WIDTH;
use crate::ui::CONFIG;
use crate::ExitStatus;
use crate::{eprintln_debug, eprintln_info, eprintln_more, eprintln_tagged, eprintln_warning};
use console::Style;
use futures::future;
use itertools::Itertools as _;
use std::cmp;

const PANE_MINIMUM_SIZE: usize = 3;

const LINE_NO_SEP: &str = " | ";
const CENTER_SEP: &str = " | ";

const EXPECTED_HEADER: &str = "<expected>";
const ACTUAL_HEADER: &str = "<actual>";

#[derive(clap::Clap)]
#[clap(about = "Runs and tests the current solution")]
pub struct Run {
    #[clap(
        short,
        long,
        help = "Recompiles even if the compiled binary seems to be up-to-date"
    )]
    force_compile: bool,
    #[clap(help = "Test case IDs to test")]
    to_run: Vec<String>,
}

fn style_output() -> Style {
    Style::new().magenta().bold()
}

fn style_sep() -> Style {
    Style::new().black().bold()
}

pub type Result<T> = std::result::Result<T, Error>;

delegate_impl_error_error_kind! {
    #[error("failed to run some test cases")]
    pub struct Error(ErrorKind);
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("failed to compile")]
    CompilationFailed { source: anyhow::Error },

    #[error("test running failed")]
    RunningTestsFailed { source: anyhow::Error },

    #[error("failed to get source file")]
    GettingLanguageFailed { source: anyhow::Error },

    #[error("failed to copy to clipboard")]
    CopyingToClipboardFailed { source: anyhow::Error },

    #[error("failed to parse the Accepted argument")]
    InvalidArgument { source: anyhow::Error },

    #[error("failed to load some test case")]
    LoadingTestCaseFailed { source: anyhow::Error },

    #[error("failed to judge")]
    JudgingFailed { source: anyhow::Error },
}

impl Run {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_language()
            .map_err(|e| Error(ErrorKind::GettingLanguageFailed { source: e.into() }))?;
        let status = compile::compile(quiet, &*lang, self.force_compile)
            .map_err(|e| Error(ErrorKind::CompilationFailed { source: e.into() }))?;
        let result = if status == ExitStatus::Success {
            async_std::task::block_on(async {
                run_tests(quiet, &*lang, &self.to_run)
                    .await
                    .map_err(|e| Error(ErrorKind::RunningTestsFailed { source: e.into() }))
            })?
        } else {
            TestResult::CompilationError
        };

        let style = result_to_style(&result);
        let long_name = result.long_name();
        eprintln!("");
        eprintln!("    Verdict: {}", style.apply_to(long_name));

        // copy the answer to the clipboard
        if result.is_accepted() {
            eprintln!("");
            clip::copy_to_clipboard(quiet, &*lang)
                .map_err(|e| Error(ErrorKind::CopyingToClipboardFailed { source: e.into() }))?;

            Ok(ExitStatus::Success)
        } else {
            Ok(ExitStatus::Failure)
        }
    }
}

async fn run_tests<L: Language + ?Sized>(
    quiet: bool,
    lang: &L,
    args: &[String],
) -> Result<TestResult> {
    let tcs = enumerate_test_cases(&args)?;
    run(quiet, lang, tcs).await
}

fn parse_argument_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let mut result = vec![];
    for arg in args.iter() {
        let n: i32 = arg
            .parse::<i32>()
            .map_err(|e| Error(ErrorKind::InvalidArgument { source: e.into() }))?;
        let tcf = TestCase::load_from_index_of(n)
            .map_err(|e| Error(ErrorKind::LoadingTestCaseFailed { source: e.into() }))?;
        result.push(tcf);
    }

    Ok(result)
}

fn enumerate_test_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let test_cases = if args.is_empty() {
        test_case::enumerate_test_cases()
            .map_err(|e| Error(ErrorKind::LoadingTestCaseFailed { source: e.into() }))?
    } else {
        parse_argument_cases(args)?
    };

    Ok(test_cases)
}

async fn run<L: Language + ?Sized>(
    quiet: bool,
    lang: &L,
    tcs: Vec<TestCase>,
) -> Result<TestResult> {
    eprintln_tagged!(
        "Running": "{} test cases (current timeout is {} millisecs)",
        tcs.len(),
        CONFIG.run.timeout_milliseconds,
    );

    let judge_results = tcs.into_iter().map(|tc| {
        let cmd = lang.run_command();
        async move { (tc.to_string(), tc.judge(cmd)) }
    });

    // wait for finish of all threads
    let judge_results = future::join_all(judge_results).await;

    eprintln_tagged!("Finished": "running");
    eprintln!("");
    let mut whole_result = TestResult::Accepted;
    for (display, judge_result) in judge_results {
        let judge_result =
            judge_result.map_err(|e| Error(ErrorKind::JudgingFailed { source: e.into() }))?;
        print_result(quiet, &judge_result, display);

        // update the whole result
        let result = judge_result.result;
        if result.is_failed() && whole_result.is_accepted() {
            whole_result = result;
        }
    }

    Ok(whole_result)
}

fn print_result(quiet: bool, judge_result: &JudgeResult, display: String) {
    let JudgeResult { result, elapsed } = judge_result;

    // get color and short result string
    let style = result_to_style(&result);
    let short_name = result.short_name();

    eprintln!(
        "    {:<3} {} (in {} ms)",
        style.apply_to(short_name),
        display,
        elapsed.as_millis()
    );

    match result {
        TestResult::WrongAnswer(wa) => {
            if !quiet {
                print_wa(wa);
            }
        }
        TestResult::RuntimeError(re) => {
            if !quiet {
                print_re(*re);
            }
        }
        _ => {}
    }
}

fn result_to_style(result: &TestResult) -> Style {
    match result {
        TestResult::Accepted => Style::new().green(),
        TestResult::WrongAnswer(_) => Style::new().yellow(),
        TestResult::PresentationError => Style::new().yellow(),
        TestResult::TimeLimitExceeded => Style::new().yellow(),
        TestResult::RuntimeError(_) => Style::new().red(),
        TestResult::CompilationError => Style::new().yellow(),
    }
}

fn print_wa(wa: &WrongAnswer) {
    let style = style_output();

    eprintln_info!("expected stdout:");
    for l in &wa.context.expected {
        eprintln_more!("{}", style.apply_to(l));
    }

    eprintln_info!("actual stdout:");
    for l in &wa.context.actual {
        eprintln_more!("{}", style.apply_to(l));
    }

    eprintln_info!("errors:");
    print_wa_errors(wa);
}

fn print_wa_errors(wa: &WrongAnswer) {
    // print error messages
    let style = style_output();
    for d in &wa.errors {
        eprintln_more!("+ {}", style.apply_to(d));
    }

    let Context {
        expected, actual, ..
    } = &wa.context;
    let (expected_spans, actual_spans): (Vec<_>, Vec<_>) = wa
        .errors
        .iter()
        .flat_map(|d| match d {
            WrongAnswerKind::NumOfTokenDiffers {
                expected_span,
                actual_span,
                ..
            } => Some((*expected_span, *actual_span)),
            WrongAnswerKind::TokenDiffers {
                expected, actual, ..
            } => Some((expected.span, actual.span)),
            _ => None,
        })
        .unzip();

    // format & print diffs
    print_diffs(expected, actual, &expected_spans, &actual_spans);
}

fn print_diffs(
    expected: &[String],
    actual: &[String],
    expected_spans: &[Span],
    actual_spans: &[Span],
) {
    use console::Term;
    use splitv::Pane;
    use std::iter::{once, repeat};

    let stderr = Term::stdout();

    // calculate minimum required width
    let max_line_no = cmp::max(expected.len(), actual.len());
    let line_no_width = max_line_no.to_string().len();
    let deco_width = TAG_WIDTH + 1 + line_no_width + LINE_NO_SEP.len() + CENTER_SEP.len();
    let least_width = deco_width + PANE_MINIMUM_SIZE * 2;

    // get terminal width
    let (_, width) = stderr.size();
    let width = width as usize;
    eprintln_debug!("The terminal width is {}", width);

    if least_width > width {
        eprintln_warning!(
            "Terminal size is too narrow (at least: {} chars per line, actual: {} chars).  Diff view is disabled.",
            least_width,
            width
        );
        return;
    }

    let half = (width - deco_width) / 2;

    let expected_len = expected.iter().map(String::len).collect_vec();
    let actual_len = actual.iter().map(String::len).collect_vec();

    let expected_len_max = cmp::max(
        EXPECTED_HEADER.len(),
        expected_len.iter().max().copied().unwrap_or(0),
    );
    let actual_len_max = cmp::max(
        ACTUAL_HEADER.len(),
        actual_len.iter().max().copied().unwrap_or(0),
    );

    let expected_pane_width = cmp::min(half, cmp::max(expected_len_max, PANE_MINIMUM_SIZE));
    let actual_pane_width = cmp::min(half, cmp::max(actual_len_max, PANE_MINIMUM_SIZE));

    let style_sep = style_sep();
    let line_no_sep = &style_sep.apply_to(LINE_NO_SEP).to_string();
    let center_sep = &style_sep.apply_to(CENTER_SEP).to_string();
    let body = {
        let line_nos = (1..=max_line_no).map(|x| x.to_string()).collect_vec();
        let line_no_pane = Pane {
            lines: &once("")
                .chain(line_nos.iter().map(String::as_str))
                .collect_vec(),
            width: line_no_width,
        };

        let expected_pane = Pane {
            lines: &once(EXPECTED_HEADER)
                .chain(expected.iter().map(String::as_str))
                .collect_vec(),
            width: expected_pane_width,
        };

        let actual_pane = Pane {
            lines: &once(ACTUAL_HEADER)
                .chain(actual.iter().map(String::as_str))
                .collect_vec(),
            width: actual_pane_width,
        };

        splitv::splitv(
            &[line_no_pane, expected_pane, actual_pane],
            &[line_no_sep, center_sep],
        )
    };

    let spans = {
        let organize_spans = |spans: &[Span]| -> Vec<Vec<Span>> {
            let mut organized = vec![Vec::new(); max_line_no];
            for span in spans {
                organized[span.line].push(*span);
            }

            organized
        };

        let span_to_ascii_art = |line_width: usize, spans: Vec<Span>| -> String {
            let mut line = " ".repeat(line_width);
            for Span {
                range: (start, end),
                ..
            } in spans
            {
                line.replace_range(start..end, &"^".repeat(end - start));
            }

            line
        };

        let line_sep = "-".repeat(line_no_width);
        let line_no_pane = Pane {
            lines: &once(line_sep.as_str())
                .chain(repeat("").take(max_line_no))
                .collect_vec(),
            width: line_no_width,
        };

        let expected_lines = organize_spans(expected_spans)
            .into_iter()
            .map(|spans| span_to_ascii_art(expected_len_max, spans))
            .collect_vec();
        let expected_sep = "-".repeat(expected_pane_width);
        let expected_pane = Pane {
            lines: &once(expected_sep.as_str())
                .chain(expected_lines.iter().map(String::as_str))
                .collect_vec(),
            width: expected_pane_width,
        };

        let actual_lines = organize_spans(actual_spans)
            .into_iter()
            .map(|spans| span_to_ascii_art(actual_len_max, spans))
            .collect_vec();
        let actual_sep = "-".repeat(actual_pane_width);
        let actual_pane = Pane {
            lines: &once(actual_sep.as_str())
                .chain(actual_lines.iter().map(String::as_str))
                .collect_vec(),
            width: actual_pane_width,
        };

        splitv::splitv(
            &[line_no_pane, expected_pane, actual_pane],
            &[LINE_NO_SEP, CENTER_SEP],
        )
        .into_iter()
        .map(|s| style_sep.apply_to(s).to_string())
        .collect_vec()
    };

    body.into_iter()
        .interleave(spans)
        .for_each(|l| eprintln_more!("{}", l));
}

fn print_re(re: RuntimeErrorKind) {
    eprintln_info!("{}", re);
}
