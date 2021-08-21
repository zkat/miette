/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use atty::Stream;
use once_cell::sync::OnceCell;

use crate::protocol::{Diagnostic, DiagnosticReportPrinter, Severity};
use crate::MietteError;

pub use default_printer::*;
pub use narratable_printer::*;
pub use theme::*;

mod default_printer;
mod narratable_printer;
mod theme;

static REPORTER: OnceCell<Box<dyn DiagnosticReportPrinter + Send + Sync + 'static>> =
    OnceCell::new();

/// Set the global [DiagnosticReportPrinter] that will be used when you report
/// using [DiagnosticReport].
pub fn set_reporter(
    reporter: impl DiagnosticReportPrinter + Send + Sync + 'static,
) -> Result<(), MietteError> {
    REPORTER
        .set(Box::new(reporter))
        .map_err(|_| MietteError::ReporterInstallFailed)
}

/// Used by [DiagnosticReport] to fetch the reporter that will be used to
/// print stuff out.
pub fn get_reporter() -> &'static (dyn DiagnosticReportPrinter + Send + Sync + 'static) {
    &**REPORTER.get_or_init(get_default_printer)
}

fn get_default_printer() -> Box<dyn DiagnosticReportPrinter + Send + Sync + 'static> {
    let fancy = if let Ok(string) = std::env::var("NO_COLOR") {
        string == "0"
    } else if let Ok(string) = std::env::var("CLICOLOR") {
        string != "0" || string == "1"
    } else {
        atty::is(Stream::Stdout) && atty::is(Stream::Stderr) && !ci_info::is_ci()
    };
    if fancy {
        Box::new(DefaultReportPrinter {
            theme: MietteTheme::default(),
        })
    } else {
        Box::new(NarratableReportPrinter)
    }
}

/// Literally what it says on the tin.
pub struct JokeReporter;

impl DiagnosticReportPrinter for JokeReporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        let sev = match diagnostic.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning) => "warning",
            Some(Severity::Advice) => "advice",
        };
        writeln!(
            f,
            "me, with {} {}: {}",
            sev,
            diagnostic,
            diagnostic
                .help()
                .unwrap_or_else(|| Box::new(&"have you tried not failing?"))
        )?;
        writeln!(
            f,
            "miette, her eyes enormous: you {} miette? you {}? oh! oh! jail for mother! jail for mother for One Thousand Years!!!!",
            diagnostic.code(),
            diagnostic.snippets().map(|snippets| {
                snippets.map(|snippet| snippet.message.map(|x| x.to_owned()))
                .collect::<Option<Vec<String>>>()
            }).flatten().map(|x| x.join(", ")).unwrap_or_else(||"try and cause miette to panic".into())
        )?;

        Ok(())
    }
}
