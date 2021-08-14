use std::fmt;

use thiserror::Error;

use crate::Diagnostic;

/// Convenience [Diagnostic] that can be used as an "anonymous" wrapper for
/// Errors. This is intended to be paired with [IntoDiagnostic].
#[derive(Debug, Error)]
#[error("{}", self.error)]
pub struct DiagnosticError {
    #[source]
    pub error: Box<dyn std::error::Error + Send + Sync + 'static>,
    pub code: String,
}

impl Diagnostic for DiagnosticError {
    fn code<'a>(&'a self) -> Box<dyn std::fmt::Display + 'a> {
        Box::new(&self.code)
    }
}

/// Utility Result type for functions that return boxed [Diagnostic]s.
pub type DiagnosticResult<T> = Result<T, Box<dyn Diagnostic + Send + Sync + 'static>>;

pub trait IntoDiagnostic<T, E> {
    /// Converts [Result]-like types that return regular errors into a
    /// `Result` that returns a [Diagnostic].
    fn into_diagnostic(self, code: &(dyn fmt::Display)) -> Result<T, DiagnosticError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> IntoDiagnostic<T, E> for Result<T, E> {
    fn into_diagnostic(self, code: &(dyn fmt::Display)) -> Result<T, DiagnosticError> {
        self.map_err(|e| DiagnosticError {
            error: Box::new(e),
            code: format!("{}", code),
        })
    }
}
