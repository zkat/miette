/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticDetail, DiagnosticReporter, Severity};

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
        let context = std::str::from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let mut column = context_data.column();
        let mut offset = detail.context.start.bytes();
        let mut line_offset = offset;
        let mut iter = context.chars().peekable();
        let mut line_str = String::new();
        let highlights = detail.highlights.as_ref();
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
                    for (label, span) in highlights {
                        if span.start.bytes() >= line_offset && span.end.bytes() < offset {
                            // Highlight only covers one line.
                            write!(indented(f), "{: <2} | ", "⫶")?;
                            write!(
                                f,
                                "{}{} ",
                                " ".repeat(span.start.bytes() - line_offset),
                                "^".repeat(span.len())
                            )?;
                            writeln!(f, "{}", label)?;
                        } else if span.start.bytes() < offset
                            && span.start.bytes() >= line_offset
                            && span.end.bytes() >= offset
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
