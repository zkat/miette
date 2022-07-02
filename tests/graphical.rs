#![cfg(feature = "fancy-no-backtrace")]

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
            .with_width(80)
            .with_footer("this is a footer".into())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else if std::env::var("NARRATED").is_ok() {
        NarratableReportHandler::new()
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else if let Ok(w) = std::env::var("REPLACE_TABS") {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(80)
            .tab_width(w.parse().expect("Invalid tab width."))
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(80)
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    };
    out
}

#[test]
fn empty_source() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (0, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    // For an empty string, the label cannot be rendered.
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
   ╰────
  help: try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
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
        highlight: (13, 8).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   👼🏼text
   ·     ───┬──
   ·        ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn single_line_with_two_tabs() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    std::env::set_var("REPLACE_TABS", "4");

    let src = "source\n\t\ttext\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │         text
   ·         ──┬─
   ·           ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn single_line_with_tab_in_middle() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    std::env::set_var("REPLACE_TABS", "4");

    let src = "source\ntext\ttext\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (12, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │ text    text
   ·         ──┬─
   ·           ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn external_source() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = Report::from(MyBad {
        highlight: (9, 4).into(),
    })
    .with_source_code(NamedSource::new("bad_file.rs", src));
    let out = fmt_report(err);
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ▲
   · ╰── this bit here
 2 │   text
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ▲
   ·   ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ────
 3 │     here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │ text
   · ──┬─
   ·   ╰── this bit here
 3 │   here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text text text text text
   ·   ──┬─ ──┬─      ──┬─
   ·     │    │         ╰── z
   ·     │    ╰── y
   ·     ╰── x
 3 │     here
   ╰────
  help: try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn multiple_same_line_highlights_with_tabs_in_middle() -> Result<(), MietteError> {
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

    std::env::set_var("REPLACE_TABS", "4");

    let src = "source\n  text text text\ttext text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (9, 4).into(),
        highlight2: (14, 4).into(),
        highlight3: (24, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text text text    text text
   ·   ──┬─ ──┬─         ──┬─
   ·     │    │            ╰── z
   ·     │    ╰── y
   ·     ╰── x
 3 │     here
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │     source
 2 │ ╭─▶   text
 3 │ ├─▶     here
   · ╰──── these two lines
   ╰────
  help: try doing it better next time?
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ ╭──▶ line1
 2 │ │╭─▶ line2
 3 │ ││   line3
 4 │ │├─▶ line4
   · │╰──── block 2
 5 │ ├──▶ line5
   · ╰───── block 1
   ╰────
  help: try doing it better next time?
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
    let expected = "oops::my::bad

  × wtf?!
  │ it broke :(
  ├─▶ something went wrong
  │\u{20}\u{20}\u{20}
  │   Here's a more detailed explanation of everything that actually went
  │   wrong because it's actually important.
  │\u{20}\u{20}\u{20}
  ╰─▶ very much went wrong
   ╭─[bad_file.rs:1:1]
 1 │ ╭──▶ line1
 2 │ │╭─▶ line2
 3 │ ││   line3
 4 │ │╰─▶ line4
 5 │ ├──▶ line5
   · ╰───── block 1
   ╰────
  help: try doing it better next time?
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
    let expected = "oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ ╭─▶ source
 2 │ ├─▶   text
   · ╰──── this bit here
 3 │ ╭─▶     here
 4 │ ├─▶ more here
   · ╰──── also this bit
   ╰────
  help: try doing it better next time?
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
fn url_links() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(
        code(oops::my::bad),
        help("try doing it better next time?"),
        url("https://example.com")
    )]
    struct MyBad;
    let err = MyBad;
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    assert!(out.contains("https://example.com"));
    assert!(out.contains("(link)"));
    assert!(out.contains("oops::my::bad"));
    Ok(())
}

#[test]
fn url_links_no_code() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(help("try doing it better next time?"), url("https://example.com"))]
    struct MyBad;
    let err = MyBad;
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    assert!(out.contains("https://example.com"));
    assert!(out.contains("(link)"));
    Ok(())
}

#[test]
fn disable_url_links() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(
        code(oops::my::bad),
        help("try doing it better next time?"),
        url("https://example.com")
    )]
    struct MyBad;
    let err = MyBad;
    let mut out = String::new();
    GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
        .with_links(false)
        .render_report(&mut out, &err)
        .unwrap();
    println!("Error: {}", out);
    assert!(out.contains("https://example.com"));
    assert!(!out.contains("(link)"));
    assert!(out.contains("oops::my::bad"));
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
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?

Error: oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │   text
   ╰────
  help: try doing it better next time?

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
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
        related: vec![InnerError {
            highlight: (0, 6).into(),
        }],
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?

Error: oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │   text
   ╰────
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn zero_length_eol_span() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("oops!")]
    #[diagnostic(severity(Error))]
    struct MyBad {
        #[source_code]
        src: NamedSource,
        #[label("This bit here")]
        bad_bit: SourceSpan,
    }
    let err = MyBad {
        src: NamedSource::new("issue", "this is the first line\nthis is the second line"),
        bad_bit: (23, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);

    let expected = r#"
  × oops!
   ╭─[issue:1:1]
 1 │ this is the first line
 2 │ this is the second line
   · ▲
   · ╰── This bit here
   ╰────
"#
    .to_string();

    assert_eq!(expected, out);
}
