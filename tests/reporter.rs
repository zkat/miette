use miette::{Diagnostic, DiagnosticReport, MietteError, SourceSpan};
use thiserror::Error;

#[test]
fn single_line_highlight() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight: ("this bit here", 9, 4).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn multiple_same_line_highlights() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight1: SourceSpan,
        #[highlight(ctx)]
        highlight2: SourceSpan,
    }

    let src = "source\n  text text text text text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight1: ("this bit here", 9, 4).into(),
        highlight2: ("also this bit", 14, 4).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn multiline_highlight_adjacent() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight: ("these two lines", 9, 11).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │     source\n 2 │ ╭─▶   text\n 3 │ ├─▶     here\n   · ╰──── these two lines\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn multiline_highlight_flyby() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight: ("this block", 0, 16).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ ╭─▶ source\n 2 │ │     text\n 3 │ ├─▶     here\n   · ╰──── this block\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn multiple_multiline_highlights_adjacent() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight1: SourceSpan,
        #[highlight(ctx)]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here\nmore here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight1: ("this bit here", 0, 10).into(),
        highlight2: ("also this bit", 20, 6).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn multiple_multiline_highlights_overlapping() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, "This is the part that broke")]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight1: SourceSpan,
        #[highlight(ctx)]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight1: ("this bit here", 0, 9).into(),
        highlight2: ("also this bit", 20, 1).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
