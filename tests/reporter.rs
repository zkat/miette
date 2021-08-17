use std::{fmt, sync::Arc};

use miette::{Diagnostic, DiagnosticReporter, DiagnosticSnippet, MietteError, MietteReporter};
use thiserror::Error;

#[derive(Error)]
#[error("oops!")]
struct MyBad {
    snippets: Vec<DiagnosticSnippet>,
}

impl fmt::Debug for MyBad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        MietteReporter.debug(self, f)
    }
}

impl Diagnostic for MyBad {
    fn code(&self) -> Box<dyn std::fmt::Display> {
        Box::new(&"oops::my::bad")
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display>> {
        Some(Box::new(&"try doing it better next time?"))
    }

    fn snippets(&self) -> Option<Box<dyn Iterator<Item = DiagnosticSnippet>>> {
        Some(Box::new(self.snippets.clone().into_iter()))
    }
}

#[test]
fn basic() -> Result<(), MietteError> {
    let err = MyBad {
        snippets: Vec::new(),
    };
    let out = format!("{:?}", err);
    assert_eq!(
        "Error[oops::my::bad]: oops!\n\n﹦try doing it better next time?\n".to_string(),
        out
    );
    Ok(())
}

#[test]
fn fancy() -> Result<(), MietteError> {
    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        snippets: vec![DiagnosticSnippet {
            message: Some("This is the part that broke".into()),
            source: Arc::new(src),
            highlights: Some(vec![("this bit here", 9, 4).into()]),
            context: ("bad_file.rs", 0, len).into(),
        }],
    };
    let out = format!("{:?}", err);
    // println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n    1  | source\n    2  |   text\n    ⫶  |   ^^^^ this bit here\n    3  |     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
