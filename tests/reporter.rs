use miette::{Diagnostic, DiagnosticReport, MietteError, SourceSpan};
use thiserror::Error;

#[test]
fn single_line_snippet() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        message: String,
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight: SourceSpan,
    }

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
    // println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
