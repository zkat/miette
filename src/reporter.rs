/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticDetail, DiagnosticReporter, Severity};

pub struct Reporter;

impl Reporter {
    fn render_detail(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        detail: &DiagnosticDetail,
    ) -> core::fmt::Result {
        use core::fmt::Write as _;
        write!(f, "\n[{}]", detail.source_name)?;
        if let Some(msg) = &detail.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(
            indented(f),
            "\n\n({}) @ line {}, col {} ",
            detail.span.label,
            detail.span.start.line + 1,
            detail.span.start.column + 1
        )?;
        if let Some(other_spans) = &detail.other_spans {
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
        Ok(())
    }
}

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
    fn debug(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(diagnostic, f);
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
