/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;
use once_cell::sync::OnceCell;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticReportPrinter, DiagnosticSnippet, Severity};
use crate::MietteError;

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
    &**REPORTER.get_or_init(|| Box::new(DefaultReportPrinter))
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

/**
Reference implementation of the [DiagnosticReportPrinter] trait. This is generally
good enough for simple use-cases, and is the default one installed with `miette`,
but you might want to implement your own if you want custom reporting for your
tool or app.
*/
pub struct DefaultReportPrinter;

impl DefaultReportPrinter {
    fn render_snippet(
        &self,
        f: &mut fmt::Formatter<'_>,
        snippet: &DiagnosticSnippet,
    ) -> fmt::Result {
        use fmt::Write as _;
        if let Some(source_name) = snippet.context.label() {
            write!(f, "[{}]", source_name)?;
        }
        if let Some(msg) = &snippet.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        let context_data = snippet
            .source
            .read_span(&snippet.context)
            .map_err(|_| fmt::Error)?;
        let context = std::str::from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let mut column = context_data.column();
        let mut offset = snippet.context.offset();
        let mut line_offset = offset;
        let mut iter = context.chars().peekable();
        let mut line_str = String::new();
        let highlights = snippet.highlights.as_ref();
        while let Some(char) = iter.next() {
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                        line += 1;
                        column = 0;
                    } else {
                        line_str.push(char);
                        column += 1;
                    }
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    line_str.push(char);
                    column += 1;
                }
            }
            if iter.peek().is_none() {
                line += 1;
            }

            if column == 0 || iter.peek().is_none() {
                writeln!(indented(f), "{: <2} | {}", line, line_str)?;
                line_str.clear();
                if let Some(highlights) = highlights {
                    for span in highlights {
                        if span.offset() >= line_offset && (span.offset() + span.len()) < offset {
                            // Highlight only covers one line.
                            write!(indented(f), "{: <2} | ", "⫶")?;
                            write!(
                                f,
                                "{}{} ",
                                " ".repeat(span.offset() - line_offset),
                                "^".repeat(span.len())
                            )?;
                            if let Some(label) = span.label() {
                                writeln!(f, "{}", label)?;
                            }
                        } else if span.offset() < offset
                            && span.offset() >= line_offset
                            && (span.offset() + span.len()) >= offset
                        {
                            // Multiline highlight.
                            todo!("Multiline highlights.");
                        }
                    }
                }
                line_offset = offset;
            }
        }
        Ok(())
    }
}

impl DiagnosticReportPrinter for DefaultReportPrinter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write as _;

        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        let sev = match diagnostic.severity() {
            Some(Severity::Error) | None => "Error",
            Some(Severity::Warning) => "Warning",
            Some(Severity::Advice) => "Advice",
        };
        writeln!(f, "{}[{}]: {}", sev, diagnostic.code(), diagnostic)?;

        if let Some(cause) = diagnostic.source() {
            writeln!(f)?;
            write!(f, "Caused by:")?;
            let multiple = cause.source().is_some();

            for (n, error) in Chain::new(cause).enumerate() {
                writeln!(f)?;
                if multiple {
                    write!(indented(f).ind(n), "{}", error)?;
                } else {
                    write!(indented(f), "{}", error)?;
                }
            }
        }

        if let Some(snippets) = diagnostic.snippets() {
            let mut pre = false;
            for snippet in snippets {
                if !pre {
                    writeln!(f)?;
                    pre = true;
                }
                self.render_snippet(f, &snippet)?;
            }
        }

        if let Some(help) = diagnostic.help() {
            writeln!(f)?;
            writeln!(f, "﹦{}", help)?;
        }

        Ok(())
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
