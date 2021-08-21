/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use once_cell::sync::OnceCell;

use crate::protocol::{Diagnostic, DiagnosticReportPrinter, Severity};
use crate::MietteError;

pub use theme::*;
pub use default_reporter::*;

mod default_reporter;
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
    &**REPORTER.get_or_init(|| {
        Box::new(DefaultReportPrinter {
            // TODO: color support detection here?
            theme: MietteTheme::default(),
        })
    })
}

/// Convenience alias. This is intended to be used as the return type for `main()`
pub type DiagnosticResult<T> = Result<T, DiagnosticReport>;

/// When used with `?`/`From`, this will wrap any Diagnostics and, when
/// formatted with `Debug`, will fetch the current [DiagnosticReportPrinter] and
/// use it to format the inner [Diagnostic].
pub struct DiagnosticReport {
    diagnostic: Box<dyn Diagnostic + Send + Sync + 'static>,
}

impl DiagnosticReport {
    /// Return a reference to the inner [Diagnostic].
    pub fn inner(&self) -> &(dyn Diagnostic + Send + Sync + 'static) {
        &*self.diagnostic
    }
}

impl std::fmt::Debug for DiagnosticReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        get_reporter().debug(&*self.diagnostic, f)
    }
}

impl<T: Diagnostic + Send + Sync + 'static> From<T> for DiagnosticReport {
    fn from(diagnostic: T) -> Self {
        DiagnosticReport {
            diagnostic: Box::new(diagnostic),
        }
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
