use std::fmt::Display;
use std::io::{self, Read};

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
pub trait Source {
    /// Get a `Read`er from a given [Source].
    fn open(&self) -> io::Result<Box<dyn Read>>;
}

/**
Details and additional context to be displayed.
 */
pub struct DiagnosticDetail {
    /// Explanation of this specific diagnostic detail.
    pub message: Option<String>,
    /// The "filename" for this diagnostic.
    pub source_name: String,
    /// A [Source] that can be used to read the actual text of a source.
    pub source: Box<dyn Source>,
    /// The primary [SourceSpan] where this diagnostic is located.
    pub span: SourceSpan,
    /// Additional [SourceSpan]s that can add secondary context.
    pub other_spans: Option<Vec<SourceSpan>>,
}

/**
Span within a [Source] with an associated message.
*/
pub struct SourceSpan {
    /// A name for the thing this SourceSpan is actually pointing to.
    pub label: String,
    /// The start of the span.
    pub start: SourceLocation,
    /// The end of the span.
    pub end: SourceLocation,
}

/**
Specific location in a [SourceSpan]
*/
pub struct SourceLocation {
    /// 0-indexed column of location.
    pub column: usize,
    /// 0-indexed line of location.
    pub line: usize,
    /// 0-indexed _character_ offset of location.
    pub offset: usize,
}
