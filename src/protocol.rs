use std::fmt::Display;

use crate::MietteError;

/**
Adds rich metadata to your Error that can be used by [DiagnosticReporter] to print
really nice and human-friendly error messages.
*/
pub trait Diagnostic: std::error::Error + Send + Sync + 'static {
    /// Unique diagnostic code that can be used to look up more information
    /// about this Diagnostic. Ideally also globally unique, and documented in
    /// the toplevel crate's documentation for easy searching. Rust path
    /// format (`foo::bar::baz`) is recommended, but more classic codes like
    /// `E0123` or Enums will work just fine.
    fn code(&self) -> &(dyn Display + 'static);

    /// Diagnostic severity. This may be used by [Reporter]s to change the
    /// display format of this diagnostic.
    fn severity(&self) -> Severity;

    /// Additional help text related to this Diagnostic. Do you have any
    /// advice for the poor soul who's just run into this issue?
    fn help(&self) -> Option<&[&str]> {
        None
    }

    /// Additional contextual details. This is typically used for adding
    /// marked-up source file output the way compilers often do.
    fn details(&self) -> Option<&[DiagnosticDetail]> {
        None
    }
}

/**
Protocol for [Diagnostic] handlers, which are responsible for actually printing out Diagnostics.

Blatantly based on [EyreHandler](https://docs.rs/eyre/0.6.5/eyre/trait.EyreHandler.html) (thanks, Jane!)
*/
pub trait DiagnosticReporter: core::any::Any + Send + Sync {
    /// Define the report format.
    fn debug(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result;

    /// Override for the `Display` format.
    fn display(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", diagnostic)?;
        Ok(())
    }
}

/**
[Diagnostic] severity. Intended to be used by [DiagnosticReporter] to change the
way different Diagnostics are displayed.
*/
#[derive(Copy, Clone, Debug)]
pub enum Severity {
    /// Critical failure. The program cannot continue.
    Error,
    /// Warning. Please take note.
    Warning,
    /// Just some help. Here's how you could be doing it better.
    Advice,
}

/**
Represents a readable source of some sort: a source file, a String, etc.
*/
pub trait Source: std::fmt::Debug + Send + Sync + 'static {
    /// Read a specific line from this source.
    fn read_span<'a>(&'a self, span: &SourceSpan)
        -> Result<Box<dyn SpanContents<'a> + '_>, MietteError>;
}

/**
Contents of a [Source] covered by [SourceSpan].

Includes line and column information to optimize highlight calculations.
*/
pub trait SpanContents<'a> {
    fn data(&self) -> &[u8];
    fn line(&self) -> usize;
    fn column(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct MietteSpanContents<'a> {
    /// Data from a [Source], in bytes.
    data: &'a [u8],
    // The 0-indexed line where the associated [SourceSpan] _starts_.
    line: usize,
    // The 0-indexed column where the associated [SourceSpan] _starts_.
    column: usize,
}

impl<'a> MietteSpanContents<'a> {
    pub fn new(data: &'a [u8], line: usize, column: usize) -> MietteSpanContents<'a> {
        MietteSpanContents { data, line, column }
    }
}

impl<'a> SpanContents<'a> for MietteSpanContents<'a> {
    fn data(&self) -> &[u8] {
        self.data
    }
    fn line(&self) -> usize {
        self.line
    }
    fn column(&self) -> usize {
        self.column
    }
}

/**
Details and additional context to be displayed.
 */
#[derive(Debug)]
pub struct DiagnosticDetail {
    /// Explanation of this specific diagnostic detail.
    pub message: Option<String>,
    /// The "filename" for this diagnostic.
    pub source_name: String,
    /// A [Source] that can be used to read the actual text of a source.
    pub source: Box<dyn Source>,
    /// The primary [SourceSpan] where this diagnostic is located.
    pub context: SourceSpan,
    /// Additional [SourceSpan]s that mark specific sections of the span, for
    /// example, to underline specific text within the larger span. They're
    /// paired with labels that should be applied to those sections.
    pub highlights: Option<Vec<(String, SourceSpan)>>,
}

/**
Span within a [Source] with an associated message.
*/
#[derive(Clone, Debug)]
pub struct SourceSpan {
    /// The start of the span.
    pub start: SourceOffset,
    /// The (exclusive) end of the span.
    pub end: SourceOffset,
}

impl SourceSpan {
    pub fn new(start: SourceOffset, end: SourceOffset) -> Self {
        assert!(
            start.bytes() <= end.bytes(),
            "Starting offset must come before the end offset."
        );
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.bytes() - self.start.bytes() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.start.bytes() == self.end.bytes()
    }
}

/**
"Raw" type for the byte offset from the beginning of a [Source].
*/
pub type ByteOffset = usize;

/**
Newtype that represents the [ByteOffset] from the beginning of a [Source]
*/
#[derive(Clone, Copy, Debug)]
pub struct SourceOffset(ByteOffset);

impl SourceOffset {
    /// Actual byte offset.
    pub fn bytes(&self) -> ByteOffset {
        self.0
    }
}

impl From<ByteOffset> for SourceOffset {
    fn from(bytes: ByteOffset) -> Self {
        SourceOffset(bytes)
    }
}
