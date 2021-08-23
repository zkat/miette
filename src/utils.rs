use std::fmt;

use thiserror::Error;

use crate::{Diagnostic, DiagnosticReport, Source};

/// Convenience alias. This is intended to be used as the return type for `main()`
pub type DiagnosticResult<T> = Result<T, DiagnosticReport>;

/// Convenience [Diagnostic] that can be used as an "anonymous" wrapper for
/// Errors. This is intended to be paired with [IntoDiagnostic].
#[derive(Debug, Error)]
#[error("{}", self.error)]
pub struct DiagnosticError {
    #[source]
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
    code: String,
}

impl DiagnosticError {
    /// Return a reference to the inner Error type.
    pub fn inner(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &*self.error
    }
}

impl Diagnostic for DiagnosticError {
    fn code<'a>(&'a self) -> Box<dyn std::fmt::Display + 'a> {
        Box::new(&self.code)
    }
}

/**
Convenience trait that adds a `.into_diagnostic()` method that converts a type to a `Result<T, DiagnosticError>`.
*/
pub trait IntoDiagnostic<T, E> {
    /// Converts [Result]-like types that return regular errors into a
    /// `Result` that returns a [Diagnostic].
    fn into_diagnostic(self, code: impl fmt::Display) -> Result<T, DiagnosticError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> IntoDiagnostic<T, E> for Result<T, E> {
    fn into_diagnostic(self, code: impl fmt::Display) -> Result<T, DiagnosticError> {
        self.map_err(|e| DiagnosticError {
            error: Box::new(e),
            code: format!("{}", code),
        })
    }
}

/// Utility struct for when you have a regular [Source] type, such as a String,
/// that doesn't implement `name`, or if you want to override the `.name()`
/// returned by the `Source`.
#[derive(Debug)]
pub struct NamedSource {
    source: Box<dyn Source + Send + Sync + 'static>,
    name: String,
}

impl NamedSource {
    /// Create a new [NamedSource] using a regular [Source] and giving it a [Source::name].
    pub fn new(name: impl AsRef<str>, source: impl Source + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
            name: name.as_ref().to_string(),
        }
    }

    /// Returns a reference the inner [Source] type for this [NamedSource].
    pub fn inner(&self) -> &(dyn Source + Send + Sync + 'static) {
        &*self.source
    }
}

impl Source for NamedSource {
    fn read_span<'a>(
        &'a self,
        span: &crate::SourceSpan,
    ) -> Result<Box<dyn crate::SpanContents + 'a>, crate::MietteError> {
        self.source.read_span(span)
    }

    fn name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}
