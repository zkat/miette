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
            .without_syntax_highlighting()
            .with_width(80)
            .tab_width(w.parse().expect("Invalid tab width."))
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .without_syntax_highlighting()
            .with_width(80)
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    };
    out
}

fn fmt_report_with_settings(
    diag: Report,
    with_settings: fn(GraphicalReportHandler) -> GraphicalReportHandler,
) -> String {
    let mut out = String::new();

    let handler = with_settings(GraphicalReportHandler::new_themed(
        GraphicalTheme::unicode_nocolor(),
    ));

    handler.render_report(&mut out, diag.as_ref()).unwrap();

    println!("Error:\n```\n{}\n```", out);

    out
}

#[test]
fn word_wrap_options() -> Result<(), MietteError> {
    // By default, a long word should not break
    let out =
        fmt_report_with_settings(Report::msg("abcdefghijklmnopqrstuvwxyz"), |handler| handler);

    let expected = "\n  × abcdefghijklmnopqrstuvwxyz\n".to_string();
    assert_eq!(expected, out);

    // A long word can break with a smaller width
    let out = fmt_report_with_settings(Report::msg("abcdefghijklmnopqrstuvwxyz"), |handler| {
        handler.with_width(10)
    });
    let expected = r#"
  × abcd
  │ efgh
  │ ijkl
  │ mnop
  │ qrst
  │ uvwx
  │ yz
"#
    .to_string();
    assert_eq!(expected, out);

    // Unless, word breaking is disabled
    let out = fmt_report_with_settings(Report::msg("abcdefghijklmnopqrstuvwxyz"), |handler| {
        handler.with_width(10).with_break_words(false)
    });
    let expected = "\n  × abcdefghijklmnopqrstuvwxyz\n".to_string();
    assert_eq!(expected, out);

    // Breaks should start at the boundary of each word if possible
    let out = fmt_report_with_settings(
        Report::msg("12 123 1234 12345 123456 1234567 1234567890"),
        |handler| handler.with_width(10),
    );
    let expected = r#"
  × 12
  │ 123
  │ 1234
  │ 1234
  │ 5
  │ 1234
  │ 56
  │ 1234
  │ 567
  │ 1234
  │ 5678
  │ 90
"#
    .to_string();
    assert_eq!(expected, out);

    // But long words should not break if word breaking is disabled
    let out = fmt_report_with_settings(
        Report::msg("12 123 1234 12345 123456 1234567 1234567890"),
        |handler| handler.with_width(10).with_break_words(false),
    );
    let expected = r#"
  × 12
  │ 123
  │ 1234
  │ 12345
  │ 123456
  │ 1234567
  │ 1234567890
"#
    .to_string();
    assert_eq!(expected, out);

    // Unless, of course, there are hyphens
    let out = fmt_report_with_settings(
        Report::msg("a-b a-b-c a-b-c-d a-b-c-d-e a-b-c-d-e-f a-b-c-d-e-f-g a-b-c-d-e-f-g-h"),
        |handler| handler.with_width(10).with_break_words(false),
    );
    let expected = r#"
  × a-b
  │ a-b-
  │ c a-
  │ b-c-
  │ d a-
  │ b-c-
  │ d-e
  │ a-b-
  │ c-d-
  │ e-f
  │ a-b-
  │ c-d-
  │ e-f-
  │ g a-
  │ b-c-
  │ d-e-
  │ f-g-
  │ h
"#
    .to_string();
    assert_eq!(expected, out);

    // Which requires an additional opt-out
    let out = fmt_report_with_settings(
        Report::msg("a-b a-b-c a-b-c-d a-b-c-d-e a-b-c-d-e-f a-b-c-d-e-f-g a-b-c-d-e-f-g-h"),
        |handler| {
            handler
                .with_width(10)
                .with_break_words(false)
                .with_word_splitter(textwrap::WordSplitter::NoHyphenation)
        },
    );
    let expected = r#"
  × a-b
  │ a-b-c
  │ a-b-c-d
  │ a-b-c-d-e
  │ a-b-c-d-e-f
  │ a-b-c-d-e-f-g
  │ a-b-c-d-e-f-g-h
"#
    .to_string();
    assert_eq!(expected, out);

    // Or if there are _other_ unicode word boundaries
    let out = fmt_report_with_settings(
        Report::msg("a/b a/b/c a/b/c/d a/b/c/d/e a/b/c/d/e/f a/b/c/d/e/f/g a/b/c/d/e/f/g/h"),
        |handler| handler.with_width(10).with_break_words(false),
    );
    let expected = r#"
  × a/b
  │ a/b/
  │ c a/
  │ b/c/
  │ d a/
  │ b/c/
  │ d/e
  │ a/b/
  │ c/d/
  │ e/f
  │ a/b/
  │ c/d/
  │ e/f/
  │ g a/
  │ b/c/
  │ d/e/
  │ f/g/
  │ h
"#
    .to_string();
    assert_eq!(expected, out);

    // Such things require you to opt-in to only breaking on ASCII whitespace
    let out = fmt_report_with_settings(
        Report::msg("a/b a/b/c a/b/c/d a/b/c/d/e a/b/c/d/e/f a/b/c/d/e/f/g a/b/c/d/e/f/g/h"),
        |handler| {
            handler
                .with_width(10)
                .with_break_words(false)
                .with_word_separator(textwrap::WordSeparator::AsciiSpace)
        },
    );
    let expected = r#"
  × a/b
  │ a/b/c
  │ a/b/c/d
  │ a/b/c/d/e
  │ a/b/c/d/e/f
  │ a/b/c/d/e/f/g
  │ a/b/c/d/e/f/g/h
"#
    .to_string();
    assert_eq!(expected, out);

    Ok(())
}

#[test]
fn wrap_option() -> Result<(), MietteError> {
    // A line should break on the width
    let out = fmt_report_with_settings(
        Report::msg("abc def ghi jkl mno pqr stu vwx yz abc def ghi jkl mno pqr stu vwx yz"),
        |handler| handler.with_width(15),
    );
    let expected = r#"
  × abc def
  │ ghi jkl
  │ mno pqr
  │ stu vwx
  │ yz abc
  │ def ghi
  │ jkl mno
  │ pqr stu
  │ vwx yz
"#
    .to_string();
    assert_eq!(expected, out);

    // Unless, wrapping is disabled
    let out = fmt_report_with_settings(
        Report::msg("abc def ghi jkl mno pqr stu vwx yz abc def ghi jkl mno pqr stu vwx yz"),
        |handler| handler.with_width(15).with_wrap_lines(false),
    );
    let expected =
        "\n  × abc def ghi jkl mno pqr stu vwx yz abc def ghi jkl mno pqr stu vwx yz\n".to_string();
    assert_eq!(expected, out);

    // Then, user-defined new lines should be preserved wrapping is disabled
    let out = fmt_report_with_settings(
      Report::msg("abc def ghi jkl mno pqr stu vwx yz\nabc def ghi jkl mno pqr stu vwx yz\nabc def ghi jkl mno pqr stu vwx yz"),
      |handler| handler.with_width(15).with_wrap_lines(false),
  );
    let expected = r#"
  × abc def ghi jkl mno pqr stu vwx yz
  │ abc def ghi jkl mno pqr stu vwx yz
  │ abc def ghi jkl mno pqr stu vwx yz
"#
    .to_string();
    assert_eq!(expected, out);

    Ok(())
}

#[test]
fn wrapping_nested_errors() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("This is the parent error, the error withhhhh the children, kiddos, pups, as it were, and so on...")]
    #[diagnostic(
        code(mama::error),
        help(
            "try doing it better next time? I mean, you could have also done better thisssss time, but no?"
        )
    )]
    struct MamaError {
        #[diagnostic_source]
        baby: BabyError,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("Wah wah: I may be small, but I'll cause a proper bout of trouble — justt try wrapping this mess of a line, buddo!")]
    #[diagnostic(
        code(baby::error),
        help(
            "it cannot be helped... woulddddddd you really want to get rid of an error that's so cute?"
        )
    )]
    struct BabyError;

    let err = MamaError { baby: BabyError };
    let out = fmt_report_with_settings(err.into(), |handler| handler.with_width(50));
    let expected = r#"mama::error

  × This is the parent error, the error withhhhh
  │ the children, kiddos, pups, as it were, and
  │ so on...
  ╰─▶ baby::error
      
        × Wah wah: I may be small, but I'll
        │ cause a proper bout of trouble — justt
        │ try wrapping this mess of a line,
        │ buddo!
        help: it cannot be helped... woulddddddd
              you really want to get rid of an
              error that's so cute?
      
  help: try doing it better next time? I mean,
        you could have also done better thisssss
        time, but no?
"#;
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn wrapping_related_errors() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("This is the parent error, the error withhhhh the children, kiddos, pups, as it were, and so on...")]
    #[diagnostic(
        code(mama::error),
        help(
            "try doing it better next time? I mean, you could have also done better thisssss time, but no?"
        )
    )]
    struct MamaError {
        #[diagnostic_source]
        baby: BrotherError,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("Welcome to the brother-error brotherhood — where all of the wee baby errors join into a formidable force")]
    #[diagnostic(code(brother::error))]
    struct BrotherError {
        #[related]
        brethren: Vec<Box<dyn Diagnostic + Send + Sync>>,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("Wah wah: I may be small, but I'll cause a proper bout of trouble — justt try wrapping this mess of a line, buddo!")]
    #[diagnostic(help(
        "it cannot be helped... woulddddddd you really want to get rid of an error that's so cute?"
    ))]
    struct BabyError;

    #[derive(Debug, Diagnostic, Error)]
    #[error("Wah wah: I may be small, but I'll cause a proper bout of trouble — justt try wrapping this mess of a line, buddo!")]
    #[diagnostic(severity(Warning))]
    struct BabyWarning;

    #[derive(Debug, Diagnostic, Error)]
    #[error("Wah wah: I may be small, but I'll cause a proper bout of trouble — justt try wrapping this mess of a line, buddo!")]
    #[diagnostic(severity(Advice))]
    struct BabyAdvice;

    let err = MamaError {
        baby: BrotherError {
            brethren: vec![BabyError.into(), BabyWarning.into(), BabyAdvice.into()],
        },
    };
    let out = fmt_report_with_settings(err.into(), |handler| handler.with_width(50));
    let expected = r#"mama::error

  × This is the parent error, the error withhhhh
  │ the children, kiddos, pups, as it were, and
  │ so on...
  ╰─▶ brother::error
      
        × Welcome to the brother-error
        │ brotherhood — where all of the wee
        │ baby errors join into a formidable
        │ force
      
      Error:
        × Wah wah: I may be small, but I'll
        │ cause a proper bout of trouble — justt
        │ try wrapping this mess of a line,
        │ buddo!
        help: it cannot be helped... woulddddddd
              you really want to get rid of an
              error that's so cute?
      
      Warning:
        ⚠ Wah wah: I may be small, but I'll
        │ cause a proper bout of trouble — justt
        │ try wrapping this mess of a line,
        │ buddo!
      
      Advice:
        ☞ Wah wah: I may be small, but I'll
        │ cause a proper bout of trouble — justt
        │ try wrapping this mess of a line,
        │ buddo!
      
  help: try doing it better next time? I mean,
        you could have also done better thisssss
        time, but no?
"#;
    assert_eq!(expected, out);
    Ok(())
}

#[test]
fn empty_source() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
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
fn multiple_spans_multiline() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("oops!")]
    #[diagnostic(severity(Error))]
    struct MyBad {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label("big")]
        big: SourceSpan,
        #[label("small")]
        small: SourceSpan,
    }
    let err = MyBad {
        src: NamedSource::new(
            "issue",
            "\
if true {
    a
} else {
    b
}",
        ),
        big: (0, 32).into(),
        small: (14, 1).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);

    let expected = r#"
  × oops!
   ╭─[issue:1:1]
 1 │ ╭─▶ if true {
 2 │ │       a
   · │       ┬
   · │       ╰── small
 3 │ │   } else {
 4 │ │       b
 5 │ ├─▶ }
   · ╰──── big
   ╰────
"#
    .to_string();

    assert_eq!(expected, out);
}

#[test]
fn single_line_highlight_span_full_line() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("oops!")]
    #[diagnostic(severity(Error))]
    struct MyBad {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label("This bit here")]
        bad_bit: SourceSpan,
    }
    let err = MyBad {
        src: NamedSource::new("issue", "source\ntext"),
        bad_bit: (7, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);

    let expected = r#"
  × oops!
   ╭─[issue:2:1]
 1 │ source
 2 │ text
   · ──┬─
   ·   ╰── This bit here
   ╰────
"#
    .to_string();

    assert_eq!(expected, out);
}

#[test]
fn single_line_with_wide_char() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:7]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    std::env::set_var("REPLACE_TABS", "4");

    let src = "source\ntext =\ttext\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (14, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:8]
 1 │ source
 2 │ text =  text
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
fn single_line_highlight_offset_end_of_line() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (6, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:7]
 1 │ source
   ·       ▲
   ·       ╰── this bit here
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
fn single_line_highlight_include_end_of_line() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 5).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:3]
 1 │ source
 2 │   text
   ·   ──┬──
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
fn single_line_highlight_include_end_of_line_crlf() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\r\n  text\r\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (10, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:3]
 1 │ source
 2 │   text
   ·   ──┬──
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
fn single_line_highlight_with_empty_span() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:1]
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
fn multiline_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here\nand\nthis\ntoo")]
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
   ╭─[bad_file.rs:2:1]
 1 │ source
 2 │ text
   · ──┬─
   ·   ╰─┤ this bit here
   ·     │ and
   ·     │ this
   ·     │ too
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
fn multiple_multi_line_labels() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "x\ny"]
        highlight1: SourceSpan,
        #[label = "z\nw"]
        highlight2: SourceSpan,
        #[label = "a\nb"]
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
   ╭─[bad_file.rs:2:3]
 1 │ source
 2 │   text text text text text
   ·   ──┬─ ──┬─      ──┬─
   ·     │    │         ╰─┤ a
   ·     │    │           │ b
   ·     │    ╰─┤ z
   ·     │      │ w
   ·     ╰─┤ x
   ·       │ y
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
fn multiple_same_line_highlights() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
fn multiline_highlight_multiline_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "these two lines\nare the problem"]
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
   ╭─[bad_file.rs:2:3]
 1 │     source
 2 │ ╭─▶   text
 3 │ ├─▶     here
   · ╰──┤ these two lines
   ·    │ are the problem
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
        src: NamedSource<String>,
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
        src: NamedSource<String>,
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
        src: NamedSource<String>,
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
        src: NamedSource<String>,
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
        src: NamedSource<String>,
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
fn url_links_with_display_text() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(
        code(oops::my::bad),
        help("try doing it better next time?"),
        url("https://example.com")
    )]
    struct MyBad;
    let err = MyBad;
    let out = fmt_report_with_settings(err.into(), |handler| {
        handler.with_link_display_text("Read the documentation")
    });

    println!("Error: {}", out);
    assert!(out.contains("https://example.com"));
    assert!(out.contains("Read the documentation"));
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
        src: NamedSource<String>,
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
   ╭─[bad_file.rs:2:3]
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
fn related_severity() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[related]
        related: Vec<MyRelated>,
    }

    #[derive(Debug, Diagnostic, Error)]
    enum MyRelated {
        #[error("oops!")]
        #[diagnostic(
            severity(Error),
            code(oops::my::related::error),
            help("try doing it better next time?")
        )]
        Error {
            #[source_code]
            src: NamedSource<String>,
            #[label("this bit here")]
            highlight: SourceSpan,
        },

        #[error("oops!")]
        #[diagnostic(
            severity(Warning),
            code(oops::my::related::warning),
            help("try doing it better next time?")
        )]
        Warning {
            #[source_code]
            src: NamedSource<String>,
            #[label("this bit here")]
            highlight: SourceSpan,
        },

        #[error("oops!")]
        #[diagnostic(
            severity(Advice),
            code(oops::my::related::advice),
            help("try doing it better next time?")
        )]
        Advice {
            #[source_code]
            src: NamedSource<String>,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src.clone()),
        highlight: (9, 4).into(),
        related: vec![
            MyRelated::Error {
                src: NamedSource::new("bad_file.rs", src.clone()),
                highlight: (0, 6).into(),
            },
            MyRelated::Warning {
                src: NamedSource::new("bad_file.rs", src.clone()),
                highlight: (0, 6).into(),
            },
            MyRelated::Advice {
                src: NamedSource::new("bad_file.rs", src),
                highlight: (0, 6).into(),
            },
        ],
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:3]
 1 │ source
 2 │   text
   ·   ──┬─
   ·     ╰── this bit here
 3 │     here
   ╰────
  help: try doing it better next time?

Error: oops::my::related::error

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │   text
   ╰────
  help: try doing it better next time?

Warning: oops::my::related::warning

  ⚠ oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │   text
   ╰────
  help: try doing it better next time?

Advice: oops::my::related::advice

  ☞ oops!
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
fn zero_length_eol_span() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("oops!")]
    #[diagnostic(severity(Error))]
    struct MyBad {
        #[source_code]
        src: NamedSource<&'static str>,
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
   ╭─[issue:2:1]
 1 │ this is the first line
 2 │ this is the second line
   · ▲
   · ╰── This bit here
   ╰────
"#
    .to_string();

    assert_eq!(expected, out);
}

#[test]
fn primary_label() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label]
        first_label: SourceSpan,
        #[label(primary, "nope")]
        second_label: SourceSpan,
    }
    let err = MyBad {
        src: NamedSource::new("issue", "this is the first line\nthis is the second line"),
        first_label: (2, 4).into(),
        second_label: (24, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);

    // line 2 should be the primary, not line 1
    let expected = r#"
  × oops!
   ╭─[issue:2:2]
 1 │ this is the first line
   ·   ────
 2 │ this is the second line
   ·  ──┬─
   ·    ╰── nope
   ╰────
"#
    .to_string();

    assert_eq!(expected, out);
}

#[test]
fn single_line_with_wide_char_unaligned_span_start() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  👼🏼text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (10, 5).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:4]
 1 │ source
 2 │   👼🏼text
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
fn single_line_with_wide_char_unaligned_span_end() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  text 👼🏼\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:3]
 1 │ source
 2 │   text 👼🏼
   ·   ───┬───
   ·      ╰── this bit here
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
fn single_line_with_wide_char_unaligned_span_empty() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
    }

    let src = "source\n  👼🏼text\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (10, 0).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = r#"oops::my::bad

  × oops!
   ╭─[bad_file.rs:2:4]
 1 │ source
 2 │   👼🏼text
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
#[cfg(feature = "syntect-highlighter")]
fn syntax_highlighter() {
    std::env::set_var("REPLACE_TABS", "4");
    #[derive(Debug, Error, Diagnostic)]
    #[error("This is an error")]
    #[diagnostic()]
    pub struct Test {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is a label")]
        src_span: SourceSpan,
    }
    let src = NamedSource::new(
        "hello_world", //NOTE: intentionally missing file extension
        "fn main() {\n    println!(\"Hello, World!\");\n}\n".to_string(),
    )
    .with_language("Rust");
    let err = Test {
        src,
        src_span: (16, 26).into(),
    };
    let mut out = String::new();
    GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
        .render_report(&mut out, &err)
        .unwrap();
    let expected = r#"  × This is an error
   ╭─[hello_world:2:5]
 1 │ fn main() {
 2 │     println!("Hello, World!");
   ·     ─────────────┬────────────
   ·                  ╰── this is a label
 3 │ }
   ╰────
"#;
    assert!(out.contains("\u{1b}[38;2;180;142;173m"));
    assert_eq!(expected, strip_ansi_escapes::strip_str(out))
}

// This test reads a line from the current source file and renders it with Rust
// syntax highlighting. The goal is to test syntax highlighting on a non-trivial
// source code example. However, if tests are running in an environment where
// source files are missing, this will cause problems. In that case, it would
// be better to use include_str!() on a sufficiently complex example file.
#[test]
#[cfg(feature = "syntect-highlighter")]
fn syntax_highlighter_on_real_file() {
    std::env::set_var("REPLACE_TABS", "4");

    #[derive(Debug, Error, Diagnostic)]
    #[error("This is an error")]
    #[diagnostic()]
    pub struct Test {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is a label")]
        src_span: SourceSpan,
    }
    // BEGIN SOURCE SNIPPET

    let (filename, line) = (file!(), line!() as usize);

    // END SOURCE SNIPPET
    // SourceSpan constants for column and length
    const CO: usize = 28;
    const LEN: usize = 27;
    let file_src = std::fs::read_to_string(&filename).unwrap();
    let offset = miette::SourceOffset::from_location(&file_src, line, CO);
    let err = Test {
        src: NamedSource::new(&filename, file_src.clone()),
        src_span: SourceSpan::new(offset, LEN.into()),
    };

    let mut out = String::new();
    GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
        .with_context_lines(1)
        .render_report(&mut out, &err)
        .unwrap();

    let expected = format!(
        r#"  × This is an error
      ╭─[{filename}:{l2}:{CO}]
 {l1} │ 
 {l2} │     let (filename, line) = (file!(), line!() as usize);
      ·                            ─────────────┬─────────────
      ·                                         ╰── this is a label
 {l3} │ 
      ╰────
"#,
        l1 = line - 1,
        l2 = line,
        l3 = line + 1
    );
    assert!(out.contains("\u{1b}[38;2;180;142;173m"));
    assert_eq!(expected, strip_ansi_escapes::strip_str(out));
}

#[test]
fn triple_adjacent_highlight() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "this bit here"]
        highlight1: SourceSpan,
        #[label = "also this bit"]
        highlight2: SourceSpan,
        #[label = "finally we got"]
        highlight3: SourceSpan,
    }

    let src = "source\n\n\n  text\n\n\n    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 6).into(),
        highlight2: (11, 4).into(),
        highlight3: (22, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │ 
 3 │ 
 4 │   text
   ·   ──┬─
   ·     ╰── also this bit
 5 │ 
 6 │ 
 7 │     here
   ·     ──┬─
   ·       ╰── finally we got
   ╰────
  help: try doing it better next time?
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn non_adjacent_highlight() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "this bit here"]
        highlight1: SourceSpan,
        #[label = "also this bit"]
        highlight2: SourceSpan,
    }

    let src = "source\n\n\n\n  text    here".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 6).into(),
        highlight2: (12, 4).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
   ╭─[bad_file.rs:1:1]
 1 │ source
   · ───┬──
   ·    ╰── this bit here
 2 │ 
   ╰────
   ╭─[bad_file.rs:5:3]
 4 │ 
 5 │   text    here
   ·   ──┬─
   ·     ╰── also this bit
   ╰────
  help: try doing it better next time?
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_bad_offset() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "1st"]
        highlight1: SourceSpan,
    }

    let src = "blabla blibli".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (50, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
  [Failed to read contents for label `1st` (offset: 50, length: 6): OutOfBounds]
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_bad_length() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label = "1st"]
        highlight1: SourceSpan,
    }

    let src = "blabla blibli".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 50).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
  [Failed to read contents for label `1st` (offset: 0, length: 50): OutOfBounds]
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_no_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label]
        highlight1: SourceSpan,
    }

    let src = "blabla blibli".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (50, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
  [Failed to read contents for label `<none>` (offset: 50, length: 6): OutOfBounds]
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_2nd_label() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(oops::my::bad), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("1st")]
        highlight1: SourceSpan,
        #[label("2nd")]
        highlight2: SourceSpan,
    }

    let src = "blabla blibli".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src),
        highlight1: (0, 6).into(),
        highlight2: (50, 6).into(),
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::bad

  × oops!
  [Failed to read contents for label `2nd` (offset: 50, length: 6): OutOfBounds]
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_inner() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops inside!")]
    #[diagnostic(code(oops::my::inner), help("help info"))]
    struct MyInner {
        #[source_code]
        src: NamedSource<String>,
        #[label("inner label")]
        inner_label: SourceSpan,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("oops outside!")]
    #[diagnostic(code(oops::my::outer), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("outer label")]
        outer_label: SourceSpan,
        #[source]
        inner: MyInner,
    }

    let src_outer = "outer source".to_string();
    let src_inner = "inner source".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src_outer),
        outer_label: (0, 6).into(),
        inner: MyInner {
            src: NamedSource::new("bad_file2.rs", src_inner),
            inner_label: (60, 6).into(),
        },
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::outer

  × oops outside!
  ╰─▶ oops inside!
   ╭─[bad_file.rs:1:1]
 1 │ outer source
   · ───┬──
   ·    ╰── outer label
   ╰────
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}

#[test]
fn invalid_span_related() -> Result<(), MietteError> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops inside!")]
    #[diagnostic(code(oops::my::inner), help("help info"))]
    struct MyRelated {
        #[source_code]
        src: NamedSource<String>,
        #[label("inner label")]
        inner_label: SourceSpan,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error("oops outside!")]
    #[diagnostic(code(oops::my::outer), help("help info"))]
    struct MyBad {
        #[source_code]
        src: NamedSource<String>,
        #[label("outer label")]
        outer_label: SourceSpan,
        #[related]
        inner: Vec<MyRelated>,
    }

    let src_outer = "outer source".to_string();
    let src_inner = "related source".to_string();
    let err = MyBad {
        src: NamedSource::new("bad_file.rs", src_outer),
        outer_label: (0, 6).into(),
        inner: vec![MyRelated {
            src: NamedSource::new("bad_file2.rs", src_inner),
            inner_label: (60, 6).into(),
        }],
    };
    let out = fmt_report(err.into());
    println!("Error: {}", out);
    let expected = "oops::my::outer

  × oops outside!
   ╭─[bad_file.rs:1:1]
 1 │ outer source
   · ───┬──
   ·    ╰── outer label
   ╰────
  help: help info

Error: oops::my::inner

  × oops inside!
  [Failed to read contents for label `inner label` (offset: 60, length: 6): OutOfBounds]
  help: help info
";
    assert_eq!(expected, &out);
    Ok(())
}
