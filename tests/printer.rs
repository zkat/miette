#![cfg(feature = "fancy")]

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
    } else {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(80)
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
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  👼🏼text\n    here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight: (9, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │   👼🏼text
   ·   ──┬──
   ·     ╰── this bit here
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
        highlight: (0, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
   · ▲
   · ╰─ this bit here
 2 │   text
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
        highlight: (9, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │   text
   ·   ▲
   ·   ╰─ this bit here
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │   text
   ·   ────
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\ntext\n  here".to_string();
    let len = src.len();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        ctx: (0, len).into(),
        highlight: (7, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │ text
   · ──┬─
   ·   ╰── this bit here
 3 │   here
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ source
 2 │   text text text text text
   ·   ──┬─ ──┬─
   ·     ╰── this bit here
   ·          ╰── also this bit
 3 │     here
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │     source
 2 │ ╭─▶   text
 3 │ ├─▶     here
   · ╰──── these two lines
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ ╭──▶ line1
 2 │ │╭─▶ line2
 3 │ ││   line3
 4 │ │├─▶ line4
   · │╰──── block 2
 5 │ ├──▶ line5
   · ╰───── block 1
   ╰───

    ‽ try doing it better next time?
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
        src: NamedSource,
        #[snippet(src, message("This is the part that broke"))]
        ctx: SourceSpan,
        #[highlight(ctx, label = "block 1")]
        highlight1: SourceSpan,
        #[highlight(ctx)]
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
        ctx: (0, len).into(),
        highlight1: (0, len).into(),
        highlight2: (10, 9).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "
────[oops::my::bad]──────────────────────────────────────────────────────

    × wtf?!
    │ it broke :(
    ├─▶ something went wrong
    │\u{20}\u{20}\u{20}
    │   Here's a more detailed explanation of everything that actually went
    │   wrong because it's actually important.
    │\u{20}\u{20}\u{20}
    ╰─▶ very much went wrong

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ ╭──▶ line1
 2 │ │╭─▶ line2
 3 │ ││   line3
 4 │ │╰─▶ line4
 5 │ ├──▶ line5
   · ╰───── block 1
   ╰───

    ‽ try doing it better next time?
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[bad_file.rs:1:1] This is the part that broke:
 1 │ ╭─▶ source
 2 │ ├─▶   text
   · ╰──── this bit here
 3 │ ╭─▶     here
 4 │ ├─▶ more here
   · ╰──── also this bit
   ╰───

    ‽ try doing it better next time?
"#
    .trim_start()
    .to_string();
    assert_eq!(expected, out);
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
    assert!(out.contains("click for details"));
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
    assert!(out.contains("click for details"));
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
    assert!(out.contains("url: https://example.com"));
    assert!(!out.contains("click for details"));
    assert!(out.contains("oops::my::bad"));
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
    println!("Error: {}", out);
    let expected = r#"
────[oops::my::bad]──────────────────────────────────────────────────────

    × oops!

   ╭───[1:1] This is the part that broke:
 1 │ source_text_here
   ╰───

    ‽ try doing it better next time?
"#
    .trim_start();
    assert_eq!(out, expected);
}
