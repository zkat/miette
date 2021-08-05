/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticReporter, DiagnosticSnippet, Severity};

/**
Reference implementation of the [DiagnosticReporter] trait. This is generally
good enough for simple use-cases, but you might want to implement your own if
you want custom reporting for your tool or app.
*/
pub struct MietteReporter;

impl DiagnosticReporter for MietteReporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }
        self.render_diagnostic(diagnostic, f)?;
        Ok(())
    }
}

impl MietteReporter {
    fn render_diagnostic(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.render_header(diagnostic, f)?;
        self.render_causes(diagnostic, f)?;

        if let Some(snippets) = diagnostic.snippets() {
            writeln!(f)?;
            for snippet in snippets {
                self.render_snippet(f, snippet)?;
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

    fn render_header(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let sev = match diagnostic.severity() {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Advice => "Advice",
        };
        write!(f, "{}[{}]: {}", sev, diagnostic.code(), diagnostic)?;
        Ok(())
    }

    fn render_causes(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        use fmt::Write as _;
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

        Ok(())
    }

    fn render_snippet_header(
        &self,
        f: &mut fmt::Formatter<'_>,
        snippet: &DiagnosticSnippet,
    ) -> fmt::Result {
        write!(f, "\n[{}]", snippet.source_name)?;
        if let Some(msg) = &snippet.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        Ok(())
    }
    fn render_snippet(
        &self,
        f: &mut fmt::Formatter<'_>,
        snippet: &DiagnosticSnippet,
    ) -> fmt::Result {
        use fmt::Write as _;

        self.render_snippet_header(f, snippet)?;

        // The "context" is the code/etc snippet itself, without all the markup.
        // We have to fetch the data itself from its Source before we do anything else.
        let context_contents = snippet
            .source
            .read_span(&snippet.context)
            .map_err(|_| fmt::Error)?;
        let context = std::str::from_utf8(context_contents.data()).expect("Bad utf8 detected");

        // The line where the context stars.
        let mut line = context_contents.line();
        // The column in that line where the context starts.
        let mut column = context_contents.column();
        // The byte offset of the _beginning_ of the context, vs the entirety of the associated Source.
        let mut offset = snippet.context.start.offset();
        // The byte offset of the beginning of the current line.
        let mut line_start_offset = offset;
        // "Highlights" are the bits that label certain sections of the snippet context.
        let highlights = snippet.highlights.as_ref();
        // String buffer for the current line.
        let mut line_str = String::new();

        // Weird loop format because we need to be able to peek the iterator
        // (to check if we're about to be done, and for handling CRLF line
        // endings)
        let mut iter = context.chars().peekable();
        while let Some(char) = iter.next() {
            // offsets are byte-based, so...
            offset += char.len_utf8();
            // This section does two things:
            // 1. It maintains the line/column/offset counts, including handling CRLF
            // 2. it shoves characters in the line buffer, minus the line endings.
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

            // We have to check for eof and update the line count, in case the
            // snippet context doesn't end in a new line.
            let eof = iter.peek().is_none();
            if eof {
                line += 1;
            }

            // Ok, we're at a brand new line! Time to render it~
            if column == 0 || eof {
                // Write out line number, separator, and actual line contents.
                writeln!(indented(f), "{: <2} | {}", line, line_str)?;
                // We don't need the line contents anymore. So truncate it.
                line_str.clear();

                // Now comes the fun part: rendering highlights!
                if let Some(highlights) = highlights {
                    for (label, span) in highlights {
                        if span.start.offset() >= line_start_offset && span.end.offset() < offset {
                            // Highlight only covers one line.
                            write!(indented(f), "{: <2} | ", "⫶")?;
                            write!(
                                f,
                                "{}{} ",
                                " ".repeat(span.start.offset() - line_start_offset),
                                "^".repeat(span.len())
                            )?;
                            writeln!(f, "{}", label)?;
                        } else if span.start.offset() < offset
                            && span.start.offset() >= line_start_offset
                            && span.end.offset() >= offset
                        {
                            // I have no idea how to do this yet?...

                            // Multiline highlight.
                            todo!("Multiline highlights.");
                        }
                    }
                }

                // Once we're fully done processing the line, we set the line
                // offset to the current total offset.
                line_start_offset = offset;
            }
        }
        Ok(())
    }
}

/// Literally what it says on the tin.
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
                .unwrap_or_else(|| Box::new(vec!["have you tried not failing?"].into_iter()))
                .collect::<Vec<&str>>()
                .join(" ")
        )?;
        writeln!(
            f,
            "miette, her eyes enormous: you {} miette? you {}? oh! oh! jail for mother! jail for mother for One Thousand Years!!!!",
            diagnostic.code(),
            diagnostic.snippets().map(|snippets| {
                snippets.iter().map(|snippet| snippet.message.as_ref().map(|m| &m[..])).collect::<Option<Vec<&str>>>()
            }).flatten().map(|x| x.join(", ")).unwrap_or_else(||"try and cause miette to panic".into())
        )?;

        Ok(())
    }
}
