use crate::imp::config::CONFIG;
use crate::imp::langs;
use crate::imp::langs::Lang;
use crate::imp::test_case;
use crate::imp::test_case::{
    Accepted, Context, JudgeResult, PresentationError, RuntimeError, Span, TestCase, TestCaseFile,
    TestCaseStdin, TestResult, TimeLimitExceeded, WrongAnswer, WrongAnswerKind,
};
use crate::ui::print_macros::TAG_WIDTH;
use crate::ui::{clip, compile};
use crate::ExitStatus;
use crate::{eprintln_debug, eprintln_info, eprintln_more, eprintln_tagged, eprintln_warning};
use anyhow::anyhow;
use anyhow::{Context as _, Result};
use console::Style;
use itertools::Itertools as _;
use std::{cmp, thread, time};

const PANE_MINIMUM_SIZE: usize = 3;

const LINE_NO_SEP: &str = " | ";
const CENTER_SEP: &str = " | ";

const EXPECTED_HEADER: &str = "<expected>";
const ACTUAL_HEADER: &str = "<actual>";

#[derive(clap::Clap)]
#[clap(about = "Runs and tests the current solution")]
pub struct Run {
    #[clap(short, long, about = "Compiles in release mode")]
    release_compile: bool,
    #[clap(
        short,
        long,
        about = "Recompiles even if the compiled binary seems to be up-to-date"
    )]
    force_compile: bool,
    #[clap(
        short,
        long = "timeout",
        about = "Override default timeout milliseconds in config.json"
    )]
    timeout_milliseconds: Option<String>,
    #[clap(about = "Test case IDs to test")]
    to_run: Vec<String>,
}

fn style_output() -> Style {
    Style::new().magenta().bold()
}

fn style_sep() -> Style {
    Style::new().black().bold()
}

impl Run {
    pub fn run(self, quiet: bool) -> Result<ExitStatus> {
        let lang = langs::guess_lang().context("failed to get language")?;
        let status = compile::compile(quiet, self.release_compile, &*lang, self.force_compile)
            .context("failed to compile")?;
        let timeout_milliseconds = self
            .timeout_milliseconds
            .map(|timeout| {
                if timeout.to_ascii_lowercase() == "inf" {
                    Ok(None)
                } else {
                    timeout
                        .parse()
                        .map(Some)
                        .context("failed to parse timeout milliseconds")
                }
            })
            .unwrap_or_else(|| Ok(Some(CONFIG.run.timeout_milliseconds)))?;
        let timeout = timeout_milliseconds.map(time::Duration::from_millis);

        let result = if status == ExitStatus::Success {
            run_tests(quiet, self.release_compile, timeout, &*lang, &self.to_run)
                .context("failed to run tests")?
        } else {
            TestResult::CompilationError
        };

        let style = result_to_style(&result);
        let long_name = result.long_name();
        eprintln!();
        eprintln!("    Verdict: {}", style.apply_to(long_name));

        // copy the answer to the clipboard
        if result.is_accepted() {
            eprintln!();
            clip::copy_to_clipboard(true, &*lang).context("failed to copy to the clipboard")?;

            Ok(ExitStatus::Success)
        } else {
            Ok(ExitStatus::Failure)
        }
    }
}

fn run_tests<L: Lang + ?Sized>(
    quiet: bool,
    release: bool,
    timeout: Option<time::Duration>,
    lang: &L,
    args: &[String],
) -> Result<TestResult> {
    let tcs = enumerate_test_cases(&args)?;
    run(quiet, release, timeout, lang, tcs)
}

fn parse_argument_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let mut result = vec![];
    for arg in args.iter() {
        if arg == "-" {
            result.push(TestCase::Stdin(TestCaseStdin))
        } else {
            let n = arg
                .parse::<i32>()
                .with_context(|| format!("argument is not a number: {}", arg))?;
            let tcf = TestCase::File(
                TestCaseFile::load_from_index(n).context("failed to load test case")?,
            );
            result.push(tcf);
        }
    }

    Ok(result)
}

fn enumerate_test_cases(args: &[String]) -> Result<Vec<TestCase>> {
    let test_cases = if args.is_empty() {
        test_case::enumerate_test_case_files()
            .context("failed to enumerate test cases")?
            .into_iter()
            .map(TestCase::File)
            .collect()
    } else {
        parse_argument_cases(args)?
    };

    Ok(test_cases)
}

fn run<L: Lang + ?Sized>(
    quiet: bool,
    release: bool,
    timeout: Option<time::Duration>,
    lang: &L,
    tcs: Vec<TestCase>,
) -> Result<TestResult> {
    let timeout_message = match timeout {
        Some(timeout) => format!("current timeout is {} millisecs", timeout.as_millis()),
        None => "current timeout is not specified (infinity)".to_string(),
    };
    eprintln_tagged!(
        "Running": "{} test cases ({})",
        tcs.len(),
        timeout_message
    );

    let handles = tcs
        .into_iter()
        .map(|tc| {
            let cmd = if release {
                lang.release_run_command()?
            } else {
                lang.run_command()?
            };

            Ok(thread::spawn(move || {
                (tc.to_string(), tc.judge(cmd, timeout))
            }))
        })
        .collect::<Result<Vec<_>>>()?; // needs collect to spawn judge

    eprintln!();
    let mut whole_result = TestResult::Accepted(Accepted::new_empty());
    for handle in handles {
        let (display, result) = handle
            .join()
            .map_err(|_| anyhow!("judge thread panicked"))
            .context("failed to judge")?;
        let result = result.context("failed to judge")?;
        print_result(quiet, &result, display);

        // update the whole result
        let result = result.result;
        if result.is_failed() && whole_result.is_accepted() {
            whole_result = result;
        }
    }

    Ok(whole_result)
}

fn print_result(quiet: bool, result: &JudgeResult, display: String) {
    let JudgeResult { result, elapsed } = result;

    // get color and short result string
    let style = result_to_style(&result);
    let short_name = result.short_name();

    eprintln!(
        "    {:<3} {} (in {} ms)",
        style.apply_to(short_name),
        display,
        elapsed.as_millis()
    );

    if !quiet {
        match result {
            TestResult::Accepted(ac) => print_ac(ac),
            TestResult::WrongAnswer(wa) => print_wa(wa),
            TestResult::PresentationError(pe) => print_pe(pe),
            TestResult::TimeLimitExceeded(tle) => print_tle(tle),
            TestResult::RuntimeError(re) => print_re(re),
            TestResult::CompilationError => (),
        }
    }
}

fn result_to_style(result: &TestResult) -> Style {
    match result {
        TestResult::Accepted(_) => Style::new().green(),
        TestResult::WrongAnswer(_) => Style::new().yellow(),
        TestResult::PresentationError(_) => Style::new().yellow(),
        TestResult::TimeLimitExceeded(_) => Style::new().yellow(),
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

    print_stderr(&wa.stderr);

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
            "Terminal size is too narrow (at least: {} chars per line, actual: {} chars). Diff view is disabled.",
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

fn print_re(re: &RuntimeError) {
    eprintln_info!("{}", re.kind);
    print_stderr(&re.stderr);
}

fn print_ac(ac: &Accepted) {
    print_stderr(&ac.stderr);
}

fn print_tle(tle: &TimeLimitExceeded) {
    print_stderr(&tle.stderr);
}

fn print_pe(pe: &PresentationError) {
    print_stderr(&pe.stderr);
}

fn print_stderr(stderr: &str) -> bool {
    if !stderr.is_empty() {
        eprintln_info!("child stderr:");
        for line in stderr.lines() {
            eprintln_more!("{}", line);
        }
        true
    } else {
        false
    }
}
