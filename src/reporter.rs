/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticDetail, DiagnosticReporter, Severity};

pub struct Reporter;

impl DiagnosticReporter for Reporter {
    fn debug(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        use core::fmt::Write as _;

        if f.alternate() {
            return core::fmt::Debug::fmt(diagnostic, f);
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
            for DiagnosticDetail {
                source_name,
                message,
                span,
                other_spans,
                ..
            } in details
            {
                write!(f, "\n[{}]", source_name)?;
                if let Some(msg) = message {
                    write!(f, " {}:", msg)?;
                }
                writeln!(
                    indented(f),
                    "\n\n({}) @ line {}, col {} ",
                    span.label,
                    span.start.line + 1,
                    span.start.column + 1
                )?;
                if let Some(other_spans) = other_spans {
                    for span in other_spans {
                        writeln!(
                            indented(f),
                            "\n{} @ line {}, col {} ",
                            span.label,
                            span.start.line + 1,
                            span.start.column + 1
                        )?;
                    }
                }
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
