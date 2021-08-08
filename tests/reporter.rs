use std::{fmt, sync::Arc};

use miette::{
    Diagnostic, DiagnosticReporter, DiagnosticSnippet, MietteError, MietteReporter,
    SourceSpan,
};
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
    fn code(&self) -> &(dyn std::fmt::Display) {
        &"oops::my::bad"
    }

    fn help(&self) -> Option<&(dyn std::fmt::Display)> {
        Some(&"try doing it better next time?")
    }

    fn snippets(&self) -> Option<&[DiagnosticSnippet]> {
        Some(&self.snippets)
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
    let err = MyBad {
        snippets: vec![DiagnosticSnippet {
            message: Some("This is the part that broke".into()),
            source_name: "bad_file.rs".into(),
            source: Box::new(src.clone()),
            highlights: Some(vec![
                ("this bit here".into(), SourceSpan {
                    start: 9.into(),
                    end: 12.into(),
                })
            ]),
            context: SourceSpan {
                start: 0.into(),
                end: (src.len() - 1).into(),
            },
        }],
    };
    let out = format!("{:?}", err);
    // println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n    1  | source\n    2  |   text\n    ⫶  |   ^^^^ this bit here\n    3  |     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
