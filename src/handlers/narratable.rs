use std::fmt;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticSnippet, Severity};
use crate::{ReportHandler, SourceSpan, SpanContents};

/**
[ReportHandler] that renders plain text and avoids extraneous graphics.
It's optimized for screen readers and braille users, but is also used in any
non-graphical environments, such as non-TTY output.
*/
#[derive(Debug, Clone)]
pub struct NarratableReportHandler;

impl NarratableReportHandler {
    /// Create a new [NarratableReportHandler]. There are no customization
    /// options.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NarratableReportHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl NarratableReportHandler {
    /// Render a [Diagnostic]. This function is mostly internal and meant to
    /// be called by the toplevel [ReportHandler] handler, but is
    /// made public to make it easier (possible) to test in isolation from
    /// global state.
    pub fn render_report(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        self.render_header(f, diagnostic)?;
        self.render_causes(f, diagnostic)?;

        if let Some(snippets) = diagnostic.snippets() {
            for snippet in snippets {
                writeln!(f)?;
                self.render_snippet(f, &snippet)?;
            }
        }

        self.render_footer(f, diagnostic)?;
        Ok(())
    }

    fn render_header(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        writeln!(f, "{}", diagnostic)?;
        let severity = match diagnostic.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning) => "warning",
            Some(Severity::Advice) => "advice",
        };
        writeln!(f, "    Diagnostic severity: {}", severity)?;
        Ok(())
    }

    fn render_causes(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        if let Some(cause) = diagnostic.source() {
            for error in Chain::new(cause) {
                writeln!(f, "    Caused by: {}", error)?;
            }
        }

        Ok(())
    }

    fn render_footer(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        if let Some(help) = diagnostic.help() {
            writeln!(f, "diagnostic help: {}", help)?;
        }
        if let Some(code) = diagnostic.code() {
            writeln!(f, "diagnostic code: {}", code)?;
        }
        if let Some(url) = diagnostic.url() {
            writeln!(f, "For more details, see {}", url)?;
        }
        Ok(())
    }

    fn render_snippet(
        &self,
        f: &mut impl fmt::Write,
        snippet: &DiagnosticSnippet<'_>,
    ) -> fmt::Result {
        let (contents, lines) = self.get_lines(snippet)?;

        write!(f, "Begin snippet")?;
        if let Some(filename) = snippet.source.name() {
            write!(f, " for {}", filename,)?;
        }
        write!(
            f,
            " starting at line {}, column {}",
            contents.line() + 1,
            contents.column() + 1
        )?;
        if let Some(message) = snippet.message.as_deref() {
            write!(f, ": {}", message)?;
        }
        writeln!(f)?;
        writeln!(f)?;

        // Highlights are the bits we're going to underline in our overall
        // snippet, and we need to do some analysis first to come up with
        // gutter size.
        let mut highlights = snippet.highlights.clone().unwrap_or_else(Vec::new);
        // sorting is your friend.
        highlights.sort_unstable_by_key(|(_, h)| h.offset());

        // Now it's time for the fun part--actually rendering everything!
        for line in &lines {
            writeln!(f, "snippet line {}: {}", line.line_number, line.text)?;
            let relevant = highlights.iter().filter(|(_, hl)| line.span_starts(hl));
            for (label, hl) in relevant {
                let contents = snippet.source.read_span(hl).map_err(|_| fmt::Error)?;
                if contents.line() + 1 == line.line_number {
                    write!(
                        f,
                        "    highlight starting at line {}, column {}",
                        contents.line() + 1,
                        contents.column() + 1
                    )?;
                    if let Some(label) = label {
                        write!(f, ": {}", label)?;
                    }
                    writeln!(f)?;
                }
            }
        }
        writeln!(f)?;
        Ok(())
    }

    fn get_lines<'a>(
        &'a self,
        snippet: &'a DiagnosticSnippet<'a>,
    ) -> Result<(Box<dyn SpanContents + 'a>, Vec<Line>), fmt::Error> {
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
        let mut lines = Vec::new();
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
                lines.push(Line {
                    line_number: line,
                    offset: line_offset,
                    text: line_str.clone(),
                });
                line_str.clear();
                line_offset = offset;
            }
        }
        Ok((context_data, lines))
    }
}

impl ReportHandler for NarratableReportHandler {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        self.render_report(f, diagnostic)
    }
}

/*
Support types
*/

struct Line {
    line_number: usize,
    offset: usize,
    text: String,
}

impl Line {
    // Does this line contain the *beginning* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_starts(&self, span: &SourceSpan) -> bool {
        span.offset() >= self.offset
    }
}
