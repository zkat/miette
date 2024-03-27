use miette::{miette, Diagnostic, LabeledSpan, Report, SourceSpan};
use std::error::Error as StdError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("outer")]
struct MyError {
    source: io::Error,
}
impl Diagnostic for MyError {}

#[test]
fn test_boxed_str_diagnostic() {
    let error = Box::<dyn Diagnostic + Send + Sync>::from("oh no!");
    let error: Report = miette!(error);
    assert_eq!("oh no!", error.to_string());
    assert_eq!(
        "oh no!",
        error
            .downcast_ref::<Box<dyn Diagnostic + Send + Sync>>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_boxed_str_stderr() {
    let error = Box::<dyn StdError + Send + Sync>::from("oh no!");
    let error: Report = miette!(error);
    assert_eq!("oh no!", error.to_string());
    assert_eq!(
        "oh no!",
        error
            .downcast_ref::<Box<dyn StdError + Send + Sync>>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_boxed_thiserror() {
    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!"),
    };
    let report: Report = miette!(error);
    assert_eq!("oh no!", report.source().unwrap().to_string());

    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!!!!"),
    };
    let error: Box<dyn Diagnostic + Send + Sync + 'static> = Box::new(error);
    let report = Report::new_boxed(error);
    assert_eq!("oh no!!!!", report.source().unwrap().to_string());
}

#[test]
fn test_boxed_miette() {
    let error: Report = miette!("oh no!").wrap_err("it failed");
    let error = miette!(error);
    assert_eq!("oh no!", error.source().unwrap().to_string());
}

#[derive(Debug)]
struct CustomDiagnostic {
    source: Option<Report>,
    related: Vec<Box<dyn Diagnostic + Send + Sync>>,
}

impl CustomDiagnostic {
    const DISPLAY: &'static str = "CustomDiagnostic display";
    const DESCRIPTION: &'static str = "CustomDiagnostic description";
    const CODE: &'static str = "A042";
    const SEVERITY: miette::Severity = miette::Severity::Advice;
    const HELP: &'static str = "CustomDiagnostic help";
    const URL: &'static str = "https://custom-diagnostic-url";
    const LABEL: &'static str = "CustomDiagnostic label";
    const SOURCE_CODE: &'static str = "this-is-some-source-code";

    fn new() -> Self {
        Self {
            source: None,
            related: Vec::new(),
        }
    }

    fn with_source<E: StdError + Send + Sync + 'static>(self, source: E) -> Self {
        let source = miette!(source);
        Self {
            source: Some(source),
            related: Vec::new(),
        }
    }

    fn with_related<D: Diagnostic + Send + Sync + 'static>(mut self, diagnostic: D) -> Self {
        self.related.push(Box::new(diagnostic));
        self
    }
}

impl std::fmt::Display for CustomDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::DISPLAY)
    }
}

impl StdError for CustomDiagnostic {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|source| source.as_ref())
    }

    fn description(&self) -> &str {
        Self::DESCRIPTION
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source.as_ref().map(|source| source.as_ref())
    }
}

impl Diagnostic for CustomDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(Self::CODE))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Advice)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(Self::HELP))
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(Self::URL))
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        let labels = miette::LabeledSpan::new(Some(Self::LABEL.to_owned()), 0, 7);
        Some(Box::new(std::iter::once(labels)))
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&Self::SOURCE_CODE)
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(
            self.related.iter().map(|d| &**d as &'a dyn Diagnostic),
        ))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.source
            .as_ref()
            .map(|source| &**source as &dyn Diagnostic)
    }
}

#[test]
fn test_boxed_custom_diagnostic() {
    fn assert_report(report: &Report) {
        assert_eq!(
            report.source().map(|source| source.to_string()),
            Some("oh no!".to_owned()),
        );
        assert_eq!(
            report.code().map(|code| code.to_string()),
            Some(CustomDiagnostic::CODE.to_owned())
        );
        assert_eq!(report.severity(), Some(CustomDiagnostic::SEVERITY));
        assert_eq!(
            report.help().map(|help| help.to_string()),
            Some(CustomDiagnostic::HELP.to_owned())
        );
        assert_eq!(
            report.url().map(|url| url.to_string()),
            Some(CustomDiagnostic::URL.to_owned())
        );
        assert_eq!(
            report.labels().map(|labels| labels.collect::<Vec<_>>()),
            Some(vec![LabeledSpan::new(
                Some(CustomDiagnostic::LABEL.to_owned()),
                0,
                7
            )]),
        );
        let span = SourceSpan::from(0..CustomDiagnostic::SOURCE_CODE.len());
        assert_eq!(
            report.source_code().map(|source_code| source_code
                .read_span(&span, None, None)
                .expect("read data from source code successfully")
                .data()
                .to_owned()),
            Some(CustomDiagnostic::SOURCE_CODE.to_owned().into_bytes())
        );
        assert_eq!(
            report.diagnostic_source().map(|source| source.to_string()),
            Some("oh no!".to_owned()),
        );
    }

    let related = CustomDiagnostic::new();
    let main_diagnostic = CustomDiagnostic::new()
        .with_source(io::Error::new(io::ErrorKind::Other, "oh no!"))
        .with_related(related);

    let report = Report::new_boxed(Box::new(main_diagnostic));
    assert_report(&report);

    let related = CustomDiagnostic::new();
    let main_diagnostic = CustomDiagnostic::new()
        .with_source(io::Error::new(io::ErrorKind::Other, "oh no!"))
        .with_related(related);
    let main_diagnostic = Box::new(main_diagnostic) as Box<dyn Diagnostic + Send + Sync + 'static>;
    let report = miette!(main_diagnostic);
    assert_report(&report);
}

#[test]
#[ignore = "I don't know why this isn't working but it needs fixing."]
fn test_boxed_sources() {
    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!"),
    };
    let error = Box::<dyn Diagnostic + Send + Sync>::from(error);
    let error: Report = miette!(error).wrap_err("it failed");
    assert_eq!("it failed", error.to_string());
    assert_eq!("outer", error.source().unwrap().to_string());
    assert_eq!(
        "oh no!",
        error
            .source()
            .expect("outer")
            .source()
            .expect("inner")
            .to_string()
    );
}
