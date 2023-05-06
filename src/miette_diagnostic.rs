use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{Diagnostic, Severity};

/// Diagnostic that can be created at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MietteDiagnostic {
    /// Displayed diagnostic description
    pub description: String,
    /// Unique diagnostic code to look up more information
    /// about this Diagnostic. Ideally also globally unique, and documented
    /// in the toplevel crate's documentation for easy searching.
    /// Rust path format (`foo::bar::baz`) is recommended, but more classic
    /// codes like `E0123` will work just fine.
    pub code: Option<String>,
    /// [`Diagnostic`] severity. Intended to be used by
    /// [`ReportHandler`](crate::ReportHandler)s to change the way different
    /// [`Diagnostic`]s are displayed. Defaults to [`Severity::Error`].
    pub severity: Severity,
}

impl Display for MietteDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

impl Error for MietteDiagnostic {}

impl Diagnostic for MietteDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.code
            .as_ref()
            .map(Box::new)
            .map(|c| c as Box<dyn Display>)
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }
}

impl MietteDiagnostic {
    /// Create a new dynamic diagnostic with the given description.
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, MietteDiagnostic, Severity};
    ///
    /// let diag = MietteDiagnostic::new("Oops, something went wrong!");
    /// assert_eq!(diag.to_string(), "Oops, something went wrong!");
    /// assert_eq!(diag.description, "Oops, something went wrong!");
    /// assert_eq!(diag.severity, Severity::Error);
    /// ```
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            code: None,
            severity: Severity::Error,
        }
    }

    /// Return new diagnostic with the given code.
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, MietteDiagnostic};
    ///
    /// let diag = MietteDiagnostic::new("Oops, something went wrong!").with_code("foo::bar::baz");
    /// assert_eq!(diag.description, "Oops, something went wrong!");
    /// assert_eq!(diag.code, Some("foo::bar::baz".to_string()));
    /// ```
    pub fn with_code(self, code: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            ..self
        }
    }

    /// Return new diagnostic with the given severity.
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, MietteDiagnostic, Severity};
    ///
    /// let diag = MietteDiagnostic::new("I warn you to stop!").with_severity(Severity::Warning);
    /// assert_eq!(diag.description, "I warn you to stop!");
    /// assert_eq!(diag.severity, Severity::Warning);
    /// ```
    pub fn with_severity(self, severity: Severity) -> Self {
        Self { severity, ..self }
    }
}
