use std::fmt;

use owo_colors::{OwoColorize, Style};

use crate::chain::Chain;
use crate::printer::theme::*;
use crate::protocol::{Diagnostic, DiagnosticReportPrinter, DiagnosticSnippet, Severity};
use crate::{SourceSpan, SpanContents};

/**
A [DiagnosticReportPrinter] that displays a given [crate::DiagnosticReport] in a quasi-graphical way, using terminal colors, unicode drawing characters, and other such things.

This is the default reporter bundled with `miette`.

This printer can be customized by using `new_themed()` and handing it a [GraphicalTheme] of your own creation (or using one of its own defaults!)

See [crate::set_printer] for more details on customizing your global printer.

## Example

```
use miette::{GraphicalReportPrinter, GraphicalTheme};
miette::set_printer(GraphicalReportPrinter::new_themed(GraphicalTheme::unicode_nocolor()));
```
*/
#[derive(Debug, Clone)]
pub struct GraphicalReportPrinter {
    pub(crate) linkify_code: bool,
    pub(crate) theme: GraphicalTheme,
}

impl GraphicalReportPrinter {
    /// Create a new [GraphicalReportPrinter] with the default
    /// [GraphicalTheme]. This will use both unicode characters and colors.
    pub fn new() -> Self {
        Self {
            linkify_code: true,
            theme: GraphicalTheme::default(),
        }
    }

    ///Create a new [GraphicalReportPrinter] with a given [GraphicalTheme].
    pub fn new_themed(theme: GraphicalTheme) -> Self {
        Self {
            linkify_code: true,
            theme,
        }
    }

    /// Disables error code linkification using [Diagnostic::url].
    pub fn without_code_linking(mut self) -> Self {
        self.linkify_code = false;
        self
    }
}

impl Default for GraphicalReportPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicalReportPrinter {
    /// Render a [Diagnostic]. This function is mostly internal and meant to
    /// be called by the toplevel [DiagnosticReportPrinter] handler, but is
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
        let (severity_style, severity_icon) = match diagnostic.severity() {
            Some(Severity::Error) | None => (self.theme.styles.error, self.theme.characters.x),
            Some(Severity::Warning) => (self.theme.styles.warning, self.theme.characters.warning),
            Some(Severity::Advice) => (self.theme.styles.advice, self.theme.characters.point_right),
        };
        write!(f, "{}", self.theme.characters.hbar.to_string().repeat(4))?;
        if self.linkify_code && diagnostic.url().is_some() {
            let url = diagnostic.url().unwrap(); // safe
            let code = format!("{} (click for details)", diagnostic.code());
            let link = format!("\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", url, code);
            write!(f, "[{}]", link.style(self.theme.styles.code))?;
        } else {
            write!(f, "[{}]", diagnostic.code().style(self.theme.styles.code))?;
        }
        writeln!(f, "{}", self.theme.characters.hbar.to_string().repeat(20),)?;
        writeln!(f)?;
        writeln!(
            f,
            "    {} {}",
            severity_icon.style(severity_style),
            diagnostic.style(severity_style)
        )?;
        Ok(())
    }

    fn render_causes(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        let severity_style = match diagnostic.severity() {
            Some(Severity::Error) | None => self.theme.styles.error,
            Some(Severity::Warning) => self.theme.styles.warning,
            Some(Severity::Advice) => self.theme.styles.advice,
        };

        if let Some(cause) = diagnostic.source() {
            let mut cause_iter = Chain::new(cause).peekable();
            while let Some(error) = cause_iter.next() {
                let char = if cause_iter.peek().is_some() {
                    self.theme.characters.lcross
                } else {
                    self.theme.characters.lbot
                };
                let msg = format!(
                    "    {}{}{} {}",
                    char, self.theme.characters.hbar, self.theme.characters.rarrow, error
                )
                .style(severity_style)
                .to_string();
                writeln!(f, "{}", msg)?;
            }
        }

        Ok(())
    }

    fn render_footer(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        if let Some(help) = diagnostic.help() {
            let help = help.style(self.theme.styles.help);
            writeln!(f)?;
            writeln!(f, "    {} {}", self.theme.characters.fyi, help)?;
        }
        Ok(())
    }

    fn render_snippet(
        &self,
        f: &mut impl fmt::Write,
        snippet: &DiagnosticSnippet<'_>,
    ) -> fmt::Result {
        let (contents, lines) = self.get_lines(snippet)?;

        // Highlights are the bits we're going to underline in our overall
        // snippet, and we need to do some analysis first to come up with
        // gutter size.
        let mut highlights = snippet.highlights.clone().unwrap_or_else(Vec::new);
        // sorting is your friend.
        highlights.sort_unstable_by_key(|(_, h)| h.offset());
        let highlights = highlights
            .into_iter()
            .zip(self.theme.styles.highlights.iter().cloned().cycle())
            .map(|((label, hl), st)| FancySpan::new(label, hl, st))
            .collect::<Vec<_>>();

        // The max number of gutter-lines that will be active at any given
        // point. We need this to figure out indentation, so we do one loop
        // over the lines to see what the damage is gonna be.
        let mut max_gutter = 0usize;
        for line in &lines {
            let mut num_highlights = 0;
            for hl in &highlights {
                if !line.span_line_only(hl) && line.span_applies(hl) {
                    num_highlights += 1;
                }
            }
            max_gutter = std::cmp::max(max_gutter, num_highlights);
        }

        // Oh and one more thing: We need to figure out how much room our line numbers need!
        let linum_width = lines[..]
            .last()
            .expect("get_lines should always return at least one line?")
            .line_number
            .to_string()
            .len();

        // Header
        write!(
            f,
            "{}{}{}",
            " ".repeat(linum_width + 2),
            self.theme.characters.ltop,
            self.theme.characters.hbar.to_string().repeat(3),
        )?;
        if let Some(source_name) = snippet.source.name() {
            let source_name = source_name.style(self.theme.styles.filename);
            write!(
                f,
                "[{}:{}:{}]",
                source_name,
                contents.line() + 1,
                contents.column() + 1
            )?;
        } else {
            write!(
                f,
                "[{}:{}]",
                contents.line() + 1,
                contents.column() + 1
            )?;
        }
        if let Some(msg) = &snippet.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;

        // Now it's time for the fun part--actually rendering everything!
        for line in &lines {
            // Line number, appropriately padded.
            self.write_linum(f, linum_width, line.line_number)?;

            // Then, we need to print the gutter, along with any fly-bys We
            // have separate gutters depending on whether we're on the actual
            // line, or on one of the "highlight lines" below it.
            self.render_line_gutter(f, max_gutter, line, &highlights)?;

            // And _now_ we can print out the line text itself!
            writeln!(f, "{}", line.text)?;

            // Next, we write all the highlights that apply to this particular line.
            let (single_line, multi_line): (Vec<_>, Vec<_>) = highlights
                .iter()
                .filter(|hl| line.span_applies(hl))
                .partition(|hl| line.span_line_only(hl));
            if !single_line.is_empty() {
                // no line number!
                self.write_no_linum(f, linum_width)?;
                // gutter _again_
                self.render_highlight_gutter(f, max_gutter, line, &highlights)?;
                self.render_single_line_highlights(
                    f,
                    line,
                    linum_width,
                    max_gutter,
                    &single_line,
                    &highlights,
                )?;
            }
            for hl in multi_line {
                if hl.label().is_some() && line.span_ends(hl) && !line.span_starts(hl) {
                    // no line number!
                    self.write_no_linum(f, linum_width)?;
                    // gutter _again_
                    self.render_highlight_gutter(f, max_gutter, line, &highlights)?;
                    self.render_multi_line_end(f, hl)?;
                }
            }
        }
        Ok(())
    }

    fn render_line_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line,
        highlights: &[FancySpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        let mut arrow = false;
        for (i, hl) in applicable.enumerate() {
            if line.span_starts(hl) {
                gutter.push_str(&chars.ltop.style(hl.style).to_string());
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i))
                        .style(hl.style)
                        .to_string(),
                );
                gutter.push_str(&chars.rarrow.style(hl.style).to_string());
                arrow = true;
                break;
            } else if line.span_ends(hl) {
                if hl.label().is_some() {
                    gutter.push_str(&chars.lcross.style(hl.style).to_string());
                } else {
                    gutter.push_str(&chars.lbot.style(hl.style).to_string());
                }
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i))
                        .style(hl.style)
                        .to_string(),
                );
                gutter.push_str(&chars.rarrow.style(hl.style).to_string());
                arrow = true;
                break;
            } else if line.span_flyby(hl) {
                gutter.push_str(&chars.vbar.style(hl.style).to_string());
            } else {
                gutter.push(' ');
            }
        }
        write!(
            f,
            "{}{}",
            gutter,
            " ".repeat(
                if arrow { 1 } else { 3 } + max_gutter.saturating_sub(gutter.chars().count())
            )
        )?;
        Ok(())
    }

    fn render_highlight_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line,
        highlights: &[FancySpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        for (i, hl) in applicable.enumerate() {
            if !line.span_line_only(hl) && line.span_ends(hl) {
                gutter.push_str(&chars.lbot.style(hl.style).to_string());
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i) + 2)
                        .style(hl.style)
                        .to_string(),
                );
                break;
            } else {
                gutter.push_str(&chars.vbar.style(hl.style).to_string());
            }
        }
        write!(f, "{:width$}", gutter, width = max_gutter + 1)?;
        Ok(())
    }

    fn write_linum(&self, f: &mut impl fmt::Write, width: usize, linum: usize) -> fmt::Result {
        write!(
            f,
            " {:width$} {} ",
            linum,
            self.theme.characters.vbar,
            width = width
        )?;
        Ok(())
    }

    fn write_no_linum(&self, f: &mut impl fmt::Write, width: usize) -> fmt::Result {
        write!(
            f,
            " {:width$} {} ",
            "",
            self.theme.characters.vbar_break,
            width = width
        )?;
        Ok(())
    }

    fn render_single_line_highlights(
        &self,
        f: &mut impl fmt::Write,
        line: &Line,
        linum_width: usize,
        max_gutter: usize,
        single_liners: &[&FancySpan],
        all_highlights: &[FancySpan],
    ) -> fmt::Result {
        let mut underlines = String::new();
        let mut highest = 0;
        let chars = &self.theme.characters;
        for hl in single_liners {
            let hl_len = std::cmp::max(1, hl.len());
            let local_offset = hl.offset() - line.offset;
            let vbar_offset = local_offset + (hl_len / 2);
            let num_left = vbar_offset - local_offset;
            let num_right = local_offset + hl_len - vbar_offset - 1;
            let start = std::cmp::max(local_offset, highest);
            let end = local_offset + hl_len;
            if start < end {
                underlines.push_str(
                    &format!(
                        "{:width$}{}{}{}",
                        "",
                        chars.underline.to_string().repeat(num_left),
                        if hl.label().is_some() {
                            chars.underbar
                        } else {
                            chars.underline
                        },
                        chars.underline.to_string().repeat(num_right),
                        width = local_offset.saturating_sub(highest),
                    )
                    .style(hl.style)
                    .to_string(),
                );
            }
            highest = std::cmp::max(highest, end);
        }
        writeln!(f, "{}", underlines)?;

        for hl in single_liners {
            if let Some(label) = hl.label() {
                self.write_no_linum(f, linum_width)?;
                self.render_highlight_gutter(f, max_gutter, line, all_highlights)?;
                let hl_len = std::cmp::max(1, hl.len());
                let local_offset = hl.offset() - line.offset;
                let vbar_offset = local_offset + (hl_len / 2);
                let num_right = local_offset + hl_len - vbar_offset - 1;
                let lines = format!(
                    "{:width$}{}{} {}",
                    " ",
                    chars.lbot,
                    chars.hbar.to_string().repeat(num_right + 1),
                    label,
                    width = vbar_offset
                );
                writeln!(f, "{}", lines.style(hl.style))?;
            }
        }
        Ok(())
    }

    fn render_multi_line_end(&self, f: &mut impl fmt::Write, hl: &FancySpan) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.theme.characters.hbar.style(hl.style),
            hl.label().unwrap_or_else(|| "".into()),
        )?;
        Ok(())
    }

    fn get_lines<'a>(
        &'a self,
        snippet: &'a DiagnosticSnippet<'_>,
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
                    length: offset - line_offset,
                    text: line_str.clone(),
                });
                line_str.clear();
                line_offset = offset;
            }
        }
        Ok((context_data, lines))
    }
}

impl DiagnosticReportPrinter for GraphicalReportPrinter {
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
    length: usize,
    text: String,
}

impl Line {
    fn span_line_only(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset && span.offset() + span.len() <= self.offset + self.length
    }

    fn span_applies(&self, span: &FancySpan) -> bool {
        // Span starts in this line
        (span.offset() >= self.offset && span.offset() <= self.offset +self.length)
        // Span passes through this line
        || (span.offset() < self.offset && span.offset() + span.len() > self.offset + self.length) //todo
        // Span ends on this line
        || (span.offset() + span.len() >= self.offset && span.offset() + span.len() <= self.offset + self.length)
    }

    // A "flyby" is a multi-line span that technically covers this line, but
    // does not begin or end within the line itself. This method is used to
    // calculate gutters.
    fn span_flyby(&self, span: &FancySpan) -> bool {
        // the span itself starts before this line's starting offset (so, in a prev line)
        span.offset() < self.offset
            // ...and it stops after this line's end.
            && span.offset() + span.len() > self.offset + self.length
    }

    // Does this line contain the *beginning* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_starts(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset
    }

    // Does this line contain the *end* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_ends(&self, span: &FancySpan) -> bool {
        span.offset() + span.len() >= self.offset
            && span.offset() + span.len() <= self.offset + self.length
    }
}

struct FancySpan {
    label: Option<String>,
    span: SourceSpan,
    style: Style,
}

impl FancySpan {
    fn new(label: Option<String>, span: SourceSpan, style: Style) -> Self {
        FancySpan { label, span, style }
    }

    fn style(&self) -> Style {
        self.style
    }

    fn label(&self) -> Option<String> {
        self.label
            .as_ref()
            .map(|l| l.style(self.style()).to_string())
    }

    fn offset(&self) -> usize {
        self.span.offset()
    }

    fn len(&self) -> usize {
        self.span.len()
    }
}
