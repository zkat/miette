/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticDetail, DiagnosticReporter, Severity, SourceSpan};

pub struct Reporter;

impl Reporter {
    fn render_detail(&self, f: &mut fmt::Formatter<'_>, detail: &DiagnosticDetail) -> fmt::Result {
        use fmt::Write as _;
        write!(f, "\n[{}]", detail.source_name)?;
        if let Some(msg) = &detail.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        let context_data = detail
            .source
            .read_span(&detail.context)
            .map_err(|_| fmt::Error)?;
        let context= std::str::from_utf8(&context_data).expect("Bad utf8 detected");
        let mut highlights = Vec::new();
        if let Some(highs) = &detail.highlights {
            for (label, SourceSpan { start, end }) in highs {
                // TODO: should be able to grab all these locations in a single pass, no?
                let start = detail.source.find_location(*start).map_err(|_| fmt::Error)?;
                let end = detail.source.find_location(*end).map_err(|_| fmt::Error)?;
                highlights.push((
                    label,
                    (start, end)
                ));
            }
        }
        for (line_num, line) in context.lines().enumerate() {
            writeln!(indented(f), "{: <2} | {}", line_num + 1, line)?;
            for (label, (start, end)) in &highlights {
                if start.line == line_num {
                    if start.line == end.line {
                        write!(indented(f), "{: <2} | ", "⫶")?;
                        write!(f, "{}{} ", " ".repeat(start.column), "^".repeat(end.column - start.column + 1))?;
                        write!(f, "{}", label)?;
                        writeln!(f)?;
                    } else {
                        todo!()
                    }
                }
            }
        }
        Ok(())
    }
}

impl DiagnosticReporter for Reporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write as _;

        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        let sev = match diagnostic.severity() {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Advice => "Advice",
        };
        write!(f, "{}[{}]: {}", sev, diagnostic.code(), diagnostic)?;

        if let Some(cause) = diagnostic.source() {
            write!(f, "\n\nCaused by:")?;
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

        if let Some(details) = diagnostic.details() {
            writeln!(f)?;
            for detail in details {
                self.render_detail(f, detail)?;
            }
        }

        if let Some(help) = diagnostic.help() {
            writeln!(f)?;
            for msg in help {
                writeln!(f, "﹦{}", msg)?;
            }
        }

        Ok(())
    }
}

pub struct JokeReporter;

impl DiagnosticReporter for JokeReporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        let sev = match diagnostic.severity() {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        };
        writeln!(
            f,
            "me, with {} {}: {}",
            sev,
            diagnostic,
            diagnostic
                .help()
                .unwrap_or_else(|| &["have you tried not failing?"])
                .join(" ")
        )?;
        writeln!(
            f,
            "miette, her eyes enormous: you {} miette? you {}? oh! oh! jail for mother! jail for mother for One Thousand Years!!!!",
            diagnostic.code(),
            diagnostic.details().map(|details| {
                details.iter().map(|detail| detail.message.clone()).collect::<Option<Vec<String>>>()
            }).flatten().map(|x| x.join(", ")).unwrap_or_else(||"try and cause miette to panic".into())
        )?;

        Ok(())
    }
}
