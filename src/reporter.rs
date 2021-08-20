/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;
use once_cell::sync::OnceCell;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticReportPrinter, DiagnosticSnippet, Severity};
use crate::{MietteError, SourceSpan};

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
    &**REPORTER.get_or_init(|| {
        Box::new(DefaultReportPrinter {
            chars: Characters::unicode(),
        })
    })
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
pub struct DefaultReportPrinter {
    chars: Characters,
}

struct Line {
    line_number: usize,
    offset: usize,
    length: usize,
    text: String,
}

impl Line {
    fn span_line_only(&self, span: &SourceSpan) -> bool {
        span.offset() >= self.offset && span.offset() + span.len() <= self.offset + self.length
    }

    fn span_applies(&self, span: &SourceSpan) -> bool {
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
    fn span_flyby(&self, span: &SourceSpan) -> bool {
        // the span itself starts before this line's starting offset (so, in a prev line)
        span.offset() < self.offset
            // ...and it stops after this line's end.
            && span.offset() + span.len() > self.offset + self.length
    }

    // Does this line contain the *beginning* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_starts(&self, span: &SourceSpan) -> bool {
        span.offset() >= self.offset
    }

    // Does this line contain the *end* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_ends(&self, span: &SourceSpan) -> bool {
        span.offset() + span.len() >= self.offset
            && span.offset() + span.len() <= self.offset + self.length
    }
}

impl DefaultReportPrinter {
    fn render_report(
        &self,
        f: &mut fmt::Formatter<'_>,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        use fmt::Write as _;
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

    fn render_snippet(
        &self,
        f: &mut fmt::Formatter<'_>,
        snippet: &DiagnosticSnippet,
    ) -> fmt::Result {
        // Boring: The Header
        if let Some(source_name) = snippet.context.label() {
            write!(f, "[{}]", source_name)?;
        }
        if let Some(msg) = &snippet.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;

        // Fun time!

        // Our actual code, line by line! Handy!
        let lines = self.get_lines(snippet)?;

        // Highlights are the bits we're going to underline in our overall
        // snippet, and we need to do some analysis first to come up with
        // gutter size.
        let mut highlights = snippet.highlights.clone().unwrap_or_else(Vec::new);
        // sorting is your friend.
        highlights.sort_unstable_by_key(|h| h.offset());

        // The max number of gutter-lines that will be active at any given
        // point. We need this to figure out indentation, so we do one loop
        // over the lines to see what the damage is gonna be.
        let mut max_gutter = 0usize;
        for line in &lines {
            let mut num_highlights = 0;
            for hl in &highlights {
                if !line.span_line_only(hl)
                    && line.span_applies(hl)
                    && (line.span_flyby(hl) || line.span_starts(hl) || line.span_ends(hl))
                {
                    num_highlights += 1;
                }
            }
            max_gutter = std::cmp::max(max_gutter, num_highlights);
        }
        // if max_gutter > 0 {
        //     // Make room for the right-arrows
        //     max_gutter += 2;
        // }

        // Oh and one more thing: We need to figure out how much room our line numbers need!
        let linum_width = lines[..]
            .last()
            .expect("get_lines should always return at least one line?")
            .line_number
            .to_string()
            .len();

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
                if line.span_ends(hl) && !line.span_starts(hl) {
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
        f: &mut fmt::Formatter<'_>,
        max_gutter: usize,
        line: &Line,
        highlights: &[SourceSpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        for (i, hl) in applicable.enumerate() {
            if line.span_starts(hl) {
                gutter.push(self.chars.ltop);
                gutter.push_str(
                    &self
                        .chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i)),
                );
                gutter.push(self.chars.rarrow);
                gutter.push(' ');
                break;
            } else if line.span_ends(hl) {
                gutter.push(self.chars.lcross);
                gutter.push_str(
                    &self
                        .chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i)),
                );
                gutter.push(self.chars.rarrow);
                gutter.push(' ');
                break;
            } else if line.span_flyby(hl) {
                gutter.push(self.chars.vbar);
            } else {
                gutter.push(' ');
            }
        }
        write!(f, "{:width$}", gutter, width = max_gutter + 3)?;
        Ok(())
    }

    fn render_highlight_gutter(
        &self,
        f: &mut fmt::Formatter<'_>,
        max_gutter: usize,
        line: &Line,
        highlights: &[SourceSpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        for (i, hl) in applicable.enumerate() {
            if !line.span_line_only(hl) && line.span_ends(hl) {
                gutter.push(self.chars.lbot);
                gutter.push_str(
                    &self
                        .chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i) + 2),
                );
                break;
            } else {
                gutter.push(self.chars.vbar);
            }
        }
        write!(f, "{:width$}", gutter, width = max_gutter + 1)?;
        Ok(())
    }

    fn write_linum(&self, f: &mut fmt::Formatter<'_>, width: usize, linum: usize) -> fmt::Result {
        write!(f, " {:width$} {} ", linum, self.chars.vbar, width = width)?;
        Ok(())
    }

    fn write_no_linum(&self, f: &mut fmt::Formatter<'_>, width: usize) -> fmt::Result {
        write!(
            f,
            " {:width$} {} ",
            "",
            self.chars.vbar_break,
            width = width
        )?;
        Ok(())
    }

    fn render_single_line_highlights(
        &self,
        f: &mut fmt::Formatter<'_>,
        line: &Line,
        linum_width: usize,
        max_gutter: usize,
        single_liners: &[&SourceSpan],
        all_highlights: &[SourceSpan],
    ) -> fmt::Result {
        let mut underlines = String::new();
        let mut highest = 0;
        for hl in single_liners {
            let local_offset = hl.offset() - line.offset;
            let vbar_offset = local_offset + (hl.len() / 2);
            let num_left = vbar_offset - local_offset;
            let num_right = local_offset + hl.len() - vbar_offset - 1;
            let start = std::cmp::max(local_offset, highest);
            let end = local_offset + hl.len();
            if start < end {
                underlines.push_str(&format!(
                    "{:width$}{}{}{}",
                    "",
                    self.chars.underline.to_string().repeat(num_left),
                    self.chars.underbar,
                    self.chars.underline.to_string().repeat(num_right),
                    width = local_offset.saturating_sub(highest),
                ));
            }
            highest = std::cmp::max(highest, end);
        }
        writeln!(f, "{}", underlines)?;

        for hl in single_liners {
            self.write_no_linum(f, linum_width)?;
            self.render_highlight_gutter(f, max_gutter, line, all_highlights)?;
            let local_offset = hl.offset() - line.offset;
            let vbar_offset = local_offset + (hl.len() / 2);
            let num_right = local_offset + hl.len() - vbar_offset - 1;
            writeln!(
                f,
                "{:width$}{}{} {}",
                " ",
                self.chars.lbot,
                self.chars.hbar.to_string().repeat(num_right + 1),
                hl.label().unwrap_or(""), // TODO: conditional label
                width = vbar_offset
            )?;
        }
        Ok(())
    }

    fn render_multi_line_end(&self, f: &mut fmt::Formatter<'_>, hl: &SourceSpan) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.chars.hbar,
            hl.label().unwrap_or("") // TODO: conditional label
        )?;
        Ok(())
    }

    fn get_lines(&self, snippet: &DiagnosticSnippet) -> Result<Vec<Line>, fmt::Error> {
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
                    length: offset - line_offset,
                    text: line_str.clone(),
                });
                line_str.clear();
                line_offset = offset;
            }
        }
        Ok(lines)
    }
}

impl DiagnosticReportPrinter for DefaultReportPrinter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        self.render_report(f, diagnostic)
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

// Taken from ariadne here: https://github.com/zesterer/ariadne/blob/e3cb394cb56ecda116a0a1caecd385a49e7f6662/src/draw.rs
pub struct Characters {
    pub hbar: char,
    pub vbar: char,
    pub xbar: char,
    pub vbar_break: char,

    pub uarrow: char,
    pub rarrow: char,

    pub ltop: char,
    pub mtop: char,
    pub rtop: char,
    pub lbot: char,
    pub rbot: char,
    pub mbot: char,

    pub lbox: char,
    pub rbox: char,

    pub lcross: char,
    pub rcross: char,

    pub underbar: char,
    pub underline: char,
}

impl Characters {
    pub fn unicode() -> Self {
        Self {
            hbar: '─',
            vbar: '│',
            xbar: '┼',
            vbar_break: '·',
            uarrow: '▲',
            rarrow: '▶',
            ltop: '╭',
            mtop: '┬',
            rtop: '╮',
            lbot: '╰',
            mbot: '┴',
            rbot: '╯',
            lbox: '[',
            rbox: ']',
            lcross: '├',
            rcross: '┤',
            underbar: '┬',
            underline: '─',
        }
    }

    pub fn ascii() -> Self {
        Self {
            hbar: '-',
            vbar: '|',
            xbar: '+',
            vbar_break: ':',
            uarrow: '^',
            rarrow: '>',
            ltop: ',',
            mtop: 'v',
            rtop: '.',
            lbot: '`',
            mbot: '^',
            rbot: '\'',
            lbox: '[',
            rbox: ']',
            lcross: '|',
            rcross: '|',
            underbar: '|',
            underline: '^',
        }
    }
}
