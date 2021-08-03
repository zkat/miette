use std::{fmt, io};

use miette::{
    Diagnostic, DiagnosticDetail, DiagnosticReporter, Reporter, Severity, SourceLocation,
    SourceSpan,
};
use thiserror::Error;

#[derive(Error)]
#[error("oops!")]
struct MyBad {
    details: Vec<DiagnosticDetail>,
}

impl fmt::Debug for MyBad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Reporter.debug(self, f)
    }
}

impl Diagnostic for MyBad {
    fn code(&self) -> &(dyn std::fmt::Display + 'static) {
        &"oops::my::bad"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn help(&self) -> Option<&[&str]> {
        Some(&["try doing it better next time?"])
    }

    fn details(&self) -> Option<&[DiagnosticDetail]> {
        Some(&self.details)
    }
}

#[test]
fn basic() -> io::Result<()> {
    let err = MyBad {
        details: Vec::new(),
    };
    let out = format!("{:?}", err);
    assert_eq!(
        "Error[oops::my::bad]: oops!\n\n﹦try doing it better next time?\n".to_string(),
        out
    );
    Ok(())
}

#[test]
fn fancy() -> io::Result<()> {
    let err = MyBad {
        details: vec![DiagnosticDetail {
            message: Some("This is the part that broke".into()),
            source_name: "bad_file.rs".into(),
            source: Box::new("source_text".to_string()),
            other_spans: None,
            span: SourceSpan {
                label: "this thing here is bad".into(),
                start: SourceLocation {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
                end: None,
            },
        }],
    };
    let out = format!("{:?}", err);
    // println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n    (this thing here is bad) @ line 1, col 1 \n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
