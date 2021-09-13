use miette::{
    Diagnostic, GraphicalReportHandler, GraphicalTheme, MietteError, NamedSource,
    NarratableReportHandler, Report, SourceSpan,
};
use thiserror::Error;

fn fmt_report(diag: Report) -> String {
    let mut out = String::new();
    // Mostly for dev purposes.
    if std::env::var("STYLE").is_ok() {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else {
        NarratableReportHandler
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    };
    out
}

#[test]
fn single_line_highlight() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight: (9, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
snippet line 2:   text
    highlight starting at line 2, column 3: this bit here
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn single_line_highlight_no_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx)]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight: (9, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
snippet line 2:   text
    highlight starting at line 2, column 3
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiple_same_line_highlights() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight1: SourceSpan,
        #[highlight(ctx, label = "also this bit")]
        highlight2: SourceSpan,
    }

    let src = "source\n  text text text text text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (9, 4).into(),
        highlight2: (14, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
snippet line 2:   text text text text text
    highlight starting at line 2, column 3: this bit here
    highlight starting at line 2, column 8: also this bit
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiline_highlight_adjacent() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "these two lines")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight: (9, 11).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
snippet line 2:   text
    highlight starting at line 2, column 3: these two lines
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiline_highlight_flyby() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "block 1")]
        highlight1: SourceSpan,
        #[highlight(ctx, label = "block 2")]
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
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (0, len).into(),
        highlight2: (10, 9).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: line1
    highlight starting at line 1, column 1: block 1
snippet line 2: line2
    highlight starting at line 2, column 5: block 2
snippet line 3: line3
snippet line 4: line4
snippet line 6: line5

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiline_highlight_no_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "block 1")]
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
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (0, len).into(),
        highlight2: (10, 9).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: line1
    highlight starting at line 1, column 1: block 1
snippet line 2: line2
    highlight starting at line 2, column 5
snippet line 3: line3
snippet line 4: line4
snippet line 6: line5

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiple_multiline_highlights_adjacent() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight1: SourceSpan,
        #[highlight(ctx, label = "also this bit")]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here\nmore here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (0, 10).into(),
        highlight2: (20, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
    highlight starting at line 1, column 1: this bit here
snippet line 2:   text
snippet line 3:     here
    highlight starting at line 3, column 7: also this bit
snippet line 4: more here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
/// Lines are overlapping, but the offsets themselves aren't, so they _look_
/// disjunct if you only look at offsets.
fn multiple_multiline_highlights_overlapping_lines() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight1: SourceSpan,
        #[highlight(ctx, label = "also this bit")]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (0, 8).into(),
        highlight2: (9, 10).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1: This is the part that broke

snippet line 1: source
    highlight starting at line 1, column 1: this bit here
snippet line 2:   text
    highlight starting at line 2, column 3: also this bit
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
/// Offsets themselves are overlapping, regardless of lines.
#[ignore]
fn multiple_multiline_highlights_overlapping_offsets() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight1: SourceSpan,
        #[highlight(ctx, label = "also this bit")]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight1: (0, 8).into(),
        highlight2: (10, 10).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
    highlight starting at line 1, column 1: this bit here
snippet line 2:   text
    highlight starting at line 2, column 4: also this bit
snippet line 3:     here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn unnamed_snippet_shows_message() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        src: String,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
    }
    let src = "source_text_here".to_string();
    let len = src.len();
    let err = MyBad {
        src,
        ctx: (0, len).into(),
    };
    let out = fmt_report(err.into());
    println!("{}", out);
    let expected = r#"
oops!
    Diagnostic severity: error

Begin snippet starting at line 1, column 1: This is the part that broke

snippet line 1: source_text_here

diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start();
    assert_eq!(out, expected);
}
