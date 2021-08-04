/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::cmp;
use std::fmt;

use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{
    Diagnostic, DiagnosticDetail, DiagnosticReporter, Severity, SourceLocation, SourceSpan,
};

pub struct Reporter;

impl Reporter {
    fn render_span(
        &self,
        f: &mut fmt::Formatter<'_>,
        detail: &DiagnosticDetail,
        span: &SourceSpan,
    ) -> fmt::Result {
        // TODO: This literally reads `detail.source` *five times*. There's gotta be a better way???
        use fmt::Write as _;
        let src = &detail.source;
        let start = src.find_location(span.start).expect("Failed to find location");
        let end = src.find_location(span.end).expect("Failed to find location");
        let context_start = start.line - cmp::min(start.line, 2);
        let context_end = end.line + 2;
        let context_span = SourceSpan::new(
            span.label.clone(),
            src.find_offset(&
            SourceLocation {
                line: context_start,
                column: 0,
            }).expect("Failed to find starting context offset"),
            src.find_offset(&SourceLocation {
                line: context_end,
                column: 0,
            }).expect("Failed to find ending context offset"),
        );
        let window = detail
            .source
            .read_span(&context_span)
            .expect("Failed to read span window");
        let window = std::str::from_utf8(&window).expect("Failed to convert span window to utf8");
        for (line_num, line) in (context_start..context_end).zip(window.lines()) {
            writeln!(indented(f), "{: <2} | {}", line_num + 1, line)?;
            if line_num == start.line {
                writeln!(indented(f), "{: <2} | {:width$}^", "", "", width=start.column + 2)?;
            }
        }
        Ok(())
    }

    fn render_detail(
        &self,
        f: &mut fmt::Formatter<'_>,
        detail: &DiagnosticDetail,
    ) -> fmt::Result {
        use fmt::Write as _;
        write!(f, "\n[{}]", detail.source_name)?;
        if let Some(msg) = &detail.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        self.render_span(f, detail, &detail.span)?;
        if let Some(other_spans) = &detail.other_spans {
            for span in other_spans {
                writeln!(indented(f), "----")?;
                self.render_span(f, detail, span)?;
            }
        }
        Ok(())
    }
}

impl DiagnosticReporter for Reporter {
    fn debug(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
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
    fn debug(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
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
