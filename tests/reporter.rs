use miette::{Diagnostic, DiagnosticReport, DiagnosticSnippet, MietteError, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("oops!")]
struct MyBad {
    message: String,
    src: String,
    ctx: SourceSpan,
    highlight: SourceSpan,
}

impl Diagnostic for MyBad {
    fn code(&self) -> Box<dyn std::fmt::Display> {
        Box::new(&"oops::my::bad")
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display>> {
        Some(Box::new(&"try doing it better next time?"))
    }

    fn snippets(&self) -> Option<Box<dyn Iterator<Item = DiagnosticSnippet> + '_>> {
        Some(Box::new(
            vec![DiagnosticSnippet {
                message: Some(self.message.as_ref()),
                source: &self.src,
                context: self.ctx.clone(),
                highlights: Some(vec![self.highlight.clone()]),
            }]
            .into_iter(),
        ))
    }
}

#[test]
fn fancy() -> Result<(), MietteError> {
    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        message: "This is the part that broke".into(),
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight: ("this bit here", 9, 4).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n    1  | source\n    2  |   text\n    ⫶  |   ^^^^ this bit here\n    3  |     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
