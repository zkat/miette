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
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text text text text text\n   ·   ──┬─ ──┬─\n   ·     ╰── this bit here\n   ·          ╰── also this bit\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
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
        highlight1: SourceSpan,
        #[highlight(ctx)]
        highlight2: SourceSpan,
    }

    let src = r#"line1
line2
line3
line4
line5
"#
    .to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: ("bad_file.rs", 0, len).into(),
        highlight1: ("block 1", 0, len).into(),
        highlight2: ("block 2", 10, 9).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ ╭──▶ line1\n 2 │ │╭─▶ line2\n 3 │ ││   line3\n 4 │ │├─▶ line4\n   · │╰──── block 2\n 6 │ ├──▶ line5\n   · ╰───── block 1\n\n﹦try doing it better next time?\n".to_string(), out);
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
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ ╭─▶ source\n 2 │ ├─▶   text\n   · ╰──── this bit here\n 3 │ ╭─▶     here\n 4 │ ├─▶ more here\n   · ╰──── also this bit\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
// TODO: This breaks because those highlights aren't "truly" overlapping (in absolute byte offset), but they ARE overlapping in lines. Need to detect the latter case better
#[ignore]
/// Lines are overlapping, but the offsets themselves aren't, so they _look_
/// disjunct if you only look at offsets.
fn multiple_multiline_highlights_overlapping_lines() -> Result<(), MietteError> {
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
        highlight1: ("this bit here", 0, 8).into(),
        highlight2: ("also this bit", 9, 10).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
/// Offsets themselves are overlapping, regardless of lines.
fn multiple_multiline_highlights_overlapping_offsets() -> Result<(), MietteError> {
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
        highlight1: ("this bit here", 0, 8).into(),
        highlight2: ("also this bit", 10, 10).into(),
    };
    let rep: DiagnosticReport = err.into();
    let out = format!("{:?}", rep);
    println!("{}", out);
    assert_eq!("Error[oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦try doing it better next time?\n".to_string(), out);
    Ok(())
}
