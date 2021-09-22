use std::fmt;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, Severity};
use crate::{LabeledSpan, MietteError, ReportHandler, SourceCode, SourceSpan, SpanContents};

/**
[ReportHandler] that renders plain text and avoids extraneous graphics.
It's optimized for screen readers and braille users, but is also used in any
non-graphical environments, such as non-TTY output.
*/
#[derive(Debug, Clone)]
pub struct NarratableReportHandler {
    context_lines: usize,
    footer: Option<String>,
}

impl NarratableReportHandler {
    /// Create a new [NarratableReportHandler]. There are no customization
    /// options.
    pub fn new() -> Self {
        Self {
            footer: None,
            context_lines: 1,
        }
    }

    /// Set the footer to be displayed at the end of the report.
    pub fn with_footer(mut self, footer: String) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Sets the number of lines of context to show around each error.
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
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
        self.render_snippets(f, diagnostic)?;
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

    fn render_snippets(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        if let Some(source) = diagnostic.source_code() {
            if let Some(labels) = diagnostic.labels() {
                let mut labels = labels.collect::<Vec<_>>();
                labels.sort_unstable_by_key(|l| l.inner().offset());
                if !labels.is_empty() {
                    let contents = labels
                        .iter()
                        .map(|label| {
                            source.read_span(label.inner(), self.context_lines, self.context_lines)
                        })
                        .collect::<Result<Vec<Box<dyn SpanContents<'_>>>, MietteError>>()
                        .map_err(|_| fmt::Error)?;
                    let mut contexts = Vec::new();
                    for (right, right_conts) in labels.iter().cloned().zip(contents.iter()) {
                        if contexts.is_empty() {
                            contexts.push((right, right_conts));
                        } else {
                            let (left, left_conts) = contexts.last().unwrap().clone();
                            let left_end = left.offset() + left.len();
                            let right_end = right.offset() + right.len();
                            if left_conts.line() + left_conts.line_count() >= right_conts.line() {
                                // The snippets will overlap, so we create one Big Chunky Boi
                                let new_span = LabeledSpan::new(
                                    left.label().map(String::from),
                                    left.offset(),
                                    if right_end >= left_end {
                                        // Right end goes past left end
                                        right_end - left.offset()
                                    } else {
                                        // right is contained inside left
                                        left.len()
                                    },
                                );
                                if source
                                    .read_span(
                                        new_span.inner(),
                                        self.context_lines,
                                        self.context_lines,
                                    )
                                    .is_ok()
                                {
                                    contexts.pop();
                                    contexts.push((
                                        new_span, // We'll throw this away later
                                        left_conts,
                                    ));
                                } else {
                                    contexts.push((right, right_conts));
                                }
                            } else {
                                contexts.push((right, right_conts));
                            }
                        }
                    }
                    for (ctx, _) in contexts {
                        self.render_context(f, source, &ctx, &labels[..])?;
                    }
                }
            }
        }
        Ok(())
    }

    fn render_context<'a>(
        &self,
        f: &mut impl fmt::Write,
        source: &'a dyn SourceCode,
        context: &LabeledSpan,
        labels: &[LabeledSpan],
    ) -> fmt::Result {
        let (contents, lines) = self.get_lines(source, context.inner())?;
        write!(f, "Begin snippet")?;
        if let Some(filename) = contents.name() {
            write!(f, " for {}", filename,)?;
        }
        writeln!(
            f,
            " starting at line {}, column {}",
            contents.line() + 1,
            contents.column() + 1
        )?;
        writeln!(f)?;
        for line in &lines {
            writeln!(f, "snippet line {}: {}", line.line_number, line.text)?;
            let relevant = labels.iter().filter(|l| line.span_starts(l.inner()));
            for label in relevant {
                let contents = source
                    .read_span(label.inner(), self.context_lines, self.context_lines)
                    .map_err(|_| fmt::Error)?;
                if contents.line() + 1 == line.line_number {
                    write!(
                        f,
                        "    label starting at line {}, column {}",
                        contents.line() + 1,
                        contents.column() + 1
                    )?;
                    if let Some(label) = label.label() {
                        write!(f, ": {}", label)?;
                    }
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }

    fn get_lines<'a>(
        &'a self,
        source: &'a dyn SourceCode,
        context_span: &'a SourceSpan,
    ) -> Result<(Box<dyn SpanContents<'a> + 'a>, Vec<Line>), fmt::Error> {
        let context_data = source
            .read_span(context_span, self.context_lines, self.context_lines)
            .map_err(|_| fmt::Error)?;
        let context = std::str::from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let mut column = context_data.column();
        let mut offset = context_data.span().offset();
        let mut line_offset = offset;
        let mut iter = context.chars().peekable();
        let mut line_str = String::new();
        let mut lines = Vec::new();
        while let Some(char) = iter.next() {
            offset += char.len_utf8();
            let mut at_end_of_file = false;
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
                    at_end_of_file = iter.peek().is_none();
                }
                '\n' => {
                    at_end_of_file = iter.peek().is_none();
                    line += 1;
                    column = 0;
                }
                _ => {
                    line_str.push(char);
                    column += 1;
                }
            }

            if iter.peek().is_none() && !at_end_of_file {
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
