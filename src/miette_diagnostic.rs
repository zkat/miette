use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{Diagnostic, LabeledSpan, Severity};

/// Diagnostic that can be created at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MietteDiagnostic {
    /// Displayed diagnostic description
    pub description: String,
    /// Unique diagnostic code to look up more information
    /// about this Diagnostic. Ideally also globally unique, and documented
    /// in the toplevel crate's documentation for easy searching.
    /// Rust path format (`foo::bar::baz`) is recommended, but more classic
    /// codes like `E0123` will work just fine
    pub code: Option<String>,
    /// [`Diagnostic`] severity. Intended to be used by
    /// [`ReportHandler`](crate::ReportHandler)s to change the way different
    /// [`Diagnostic`]s are displayed. Defaults to [`Severity::Error`]
    pub severity: Option<Severity>,
    /// Additional help text related to this Diagnostic
    pub help: Option<String>,
    /// URL to visit for a more detailed explanation/help about this
    /// [`Diagnostic`].
    pub url: Option<String>,
    /// Labels to apply to this `Diagnostic`'s [`Diagnostic::source_code`]
    pub labels: Option<Vec<LabeledSpan>>,
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
        self.severity
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help
            .as_ref()
            .map(Box::new)
            .map(|c| c as Box<dyn Display>)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.url
            .as_ref()
            .map(Box::new)
            .map(|c| c as Box<dyn Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.labels
            .as_ref()
            .map(|ls| ls.iter().cloned())
            .map(Box::new)
            .map(|b| b as Box<dyn Iterator<Item = LabeledSpan>>)
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
    /// ```
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            labels: None,
            severity: None,
            code: None,
            help: None,
            url: None,
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
    /// assert_eq!(diag.severity, Some(Severity::Warning));
    /// ```
    pub fn with_severity(self, severity: Severity) -> Self {
        Self {
            severity: Some(severity),
            ..self
        }
    }

    /// Return new diagnostic with the given help message.
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, MietteDiagnostic};
    ///
    /// let diag = MietteDiagnostic::new("PC is not working").with_help("Try to reboot it again");
    /// assert_eq!(diag.description, "PC is not working");
    /// assert_eq!(diag.help, Some("Try to reboot it again".to_string()));
    /// ```
    pub fn with_help(self, help: impl Into<String>) -> Self {
        Self {
            help: Some(help.into()),
            ..self
        }
    }

    /// Return new diagnostic with the given URL.
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, MietteDiagnostic};
    ///
    /// let diag = MietteDiagnostic::new("PC is not working")
    ///     .with_url("https://letmegooglethat.com/?q=Why+my+pc+doesn%27t+work");
    /// assert_eq!(diag.description, "PC is not working");
    /// assert_eq!(
    ///     diag.url,
    ///     Some("https://letmegooglethat.com/?q=Why+my+pc+doesn%27t+work".to_string())
    /// );
    /// ```
    pub fn with_url(self, url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..self
        }
    }

    /// Return new diagnostic with the given label.
    ///
    /// Discards previous labels
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, LabeledSpan, MietteDiagnostic};
    ///
    /// let source = "cpp is the best language";
    ///
    /// let label = LabeledSpan::at(0..3, "This should be Rust");
    /// let diag = MietteDiagnostic::new("Wrong best language").with_label(label.clone());
    /// assert_eq!(diag.description, "Wrong best language");
    /// assert_eq!(diag.labels, Some(vec![label]));
    /// ```
    pub fn with_label(self, label: impl Into<LabeledSpan>) -> Self {
        Self {
            labels: Some(vec![label.into()]),
            ..self
        }
    }

    /// Return new diagnostic with the given labels.
    ///
    /// Discards previous labels
    ///
    /// # Examples
    /// ```
    /// use miette::{Diagnostic, LabeledSpan, MietteDiagnostic};
    ///
    /// let source = "helo wrld";
    ///
    /// let labels = vec![
    ///     LabeledSpan::at_offset(3, "add 'l'"),
    ///     LabeledSpan::at_offset(6, "add 'r'"),
    /// ];
    /// let diag = MietteDiagnostic::new("Typos in 'hello world'").with_labels(labels.clone());
    /// assert_eq!(diag.description, "Typos in 'hello world'");
    /// assert_eq!(diag.labels, Some(labels));
    /// ```
    pub fn with_labels(self, labels: Vec<LabeledSpan>) -> Self {
        Self {
            labels: Some(labels),
            ..self
        }
    }
}
