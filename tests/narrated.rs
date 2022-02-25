#![cfg(feature = "fancy")]

use miette::{Diagnostic, MietteError, NamedSource, NarratableReportHandler, Report, SourceSpan};

use miette::{GraphicalReportHandler, GraphicalTheme};

use thiserror::Error;

fn fmt_report(diag: Report) -> String {
    let mut out = String::new();
    // Mostly for dev purposes.
    if cfg!(feature = "fancy") && std::env::var("STYLE").is_ok() {
        #[cfg(feature = "fancy")]
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else {
        NarratableReportHandler::new()
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    };
    out
}

#[test]
fn single_line_with_wide_char() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  👼🏼text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   👼🏼text
    label at line 2, columns 3 to 6: this bit here
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
fn single_line_highlight() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label at line 2, columns 3 to 6: this bit here
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
fn single_line_highlight_offset_zero() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (0, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
    label at line 1, column 1: this bit here
snippet line 2:   text
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn single_line_highlight_with_empty_span() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label at line 2, column 3: this bit here
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
        #[source_code]
        src: NamedSource,
        #[label]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label at line 2, columns 3 to 6
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
fn single_line_highlight_at_line_start() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\ntext\n  here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (7, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2: text
    label at line 2, columns 1 to 4: this bit here
snippet line 3:   here
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
        #[source_code]
        src: NamedSource,
        #[label = "x"]
        highlight1: SourceSpan,
        #[label = "y"]
        highlight2: SourceSpan,
        #[label = "z"]
        highlight3: SourceSpan,
    }

    let src = "source\n  text text text text text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (9, 4).into(),
        highlight2: (14, 4).into(),
        highlight3: (24, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text text text text text
    label at line 2, columns 3 to 6: x
    label at line 2, columns 8 to 11: y
    label at line 2, columns 18 to 21: z
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
        #[source_code]
        src: NamedSource,
        #[label = "these two lines"]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 11).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label starting at line 2, column 3: these two lines
snippet line 3:     here
    label ending at line 3, column 6: these two lines
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
        #[source_code]
        src: NamedSource,
        #[label = "block 1"]
        highlight1: SourceSpan,
        #[label = "block 2"]
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
        highlight1: (0, len).into(),
        highlight2: (10, 9).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: line1
    label starting at line 1, column 1: block 1
snippet line 2: line2
    label starting at line 2, column 5: block 2
snippet line 3: line3
snippet line 4: line4
    label ending at line 4, column 1: block 2
snippet line 5: line5
    label ending at line 5, column 5: block 1
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
    #[error("wtf?!\nit broke :(")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source]
        source: Inner,
        #[source_code]
        src: NamedSource,
        #[label = "block 1"]
        highlight1: SourceSpan,
        #[label]
        highlight2: SourceSpan,
    }

    #[derive(Debug, Error)]
    #[error("something went wrong\n\nHere's a more detailed explanation of everything that actually went wrong because it's actually important.\n")]
    struct Inner(#[source] InnerInner);

    #[derive(Debug, Error)]
    #[error("very much went wrong")]
    struct InnerInner;

    let src = r#"line1
line2
line3
line4
line5
"#
    .to_string();
    let len = src.len();
    let err = MyBad {
        source: Inner(InnerInner),
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, len).into(),
        highlight2: (10, 9).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "wtf?!
it broke :(
    Diagnostic severity: error
    Caused by: something went wrong

Here's a more detailed explanation of everything that actually went wrong because it's actually important.

    Caused by: very much went wrong
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: line1
    label starting at line 1, column 1: block 1
snippet line 2: line2
    label starting at line 2, column 5
snippet line 3: line3
snippet line 4: line4
    label ending at line 4, column 1
snippet line 5: line5
    label ending at line 5, column 5: block 1
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"
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
        #[source_code]
        src: NamedSource,
        #[label = "this bit here"]
        highlight1: SourceSpan,
        #[label = "also this bit"]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here\nmore here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 10).into(),
        highlight2: (20, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
    label starting at line 1, column 1: this bit here
snippet line 2:   text
    label ending at line 2, column 3: this bit here
snippet line 3:     here
    label starting at line 3, column 7: also this bit
snippet line 4: more here
    label ending at line 4, column 3: also this bit
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad
"
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
// TODO: This breaks because those highlights aren't "truly" overlapping (in absolute byte offset),
// but they ARE overlapping in lines. Need to detect the latter case better
#[ignore]
/// Lines are overlapping, but the offsets themselves aren't, so they _look_
/// disjunct if you only look at offsets.
fn multiple_multiline_highlights_overlapping_lines() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label = "this bit here"]
        highlight1: SourceSpan,
        #[label = "also this bit"]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 8).into(),
        highlight2: (9, 10).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    assert_eq!("Error [oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦ try doing it better next time?\n".to_string(), out);
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
        #[source_code]
        src: NamedSource,
        #[label = "this bit here"]
        highlight1: SourceSpan,
        #[label = "also this bit"]
        highlight2: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 8).into(),
        highlight2: (10, 10).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    assert_eq!("Error [oops::my::bad]: oops!\n\n[bad_file.rs] This is the part that broke:\n\n 1 │ source\n 2 │   text\n   ·   ──┬─\n   ·     ╰── this bit here\n 3 │     here\n\n﹦ try doing it better next time?\n".to_string(), out);
    Ok(())
}

#[test]
fn url() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(help("try doing it better next time?"), url("https://example.com"))]
    struct MyBad;
    let err = MyBad;
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    assert!(out.contains("https://example.com"));
    Ok(())
}

#[test]
fn related() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[related]
        related: Vec<MyBad>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src.clone()),
        highlight: (9, 4).into(),
        related: vec![MyBad {
            src: NamedSource::new("bad_file.rs", src),
            highlight: (0, 6).into(),
            related: vec![],
        }],
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label at line 2, columns 3 to 6: this bit here
snippet line 3:     here
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad

Error: oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
    label at line 1, columns 1 to 6: this bit here
snippet line 2:   text
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad

"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn related_source_code_propagation() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[related]
        related: Vec<InnerError>,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad))]
    struct InnerError {
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src.clone()),
        highlight: (9, 4).into(),
        related: vec![InnerError {
            highlight: (0, 6).into(),
        }],
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops!
    Diagnostic severity: error
Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
snippet line 2:   text
    label at line 2, columns 3 to 6: this bit here
snippet line 3:     here
diagnostic help: try doing it better next time?
diagnostic code: oops::my::bad

Error: oops!
    Diagnostic severity: error

Begin snippet for bad_file.rs starting at line 1, column 1

snippet line 1: source
    label at line 1, columns 1 to 6: this bit here
snippet line 2:   text
diagnostic code: oops::my::bad
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}
