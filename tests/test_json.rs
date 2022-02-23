mod json_report_handler {

    use miette::{Diagnostic, MietteError, NamedSource, Report, SourceSpan};

    use miette::JSONReportHandler;

    use thiserror::Error;

    fn fmt_report(diag: Report) -> String {
        let mut out = String::new();
        JSONReportHandler::new()
            .render_report(&mut out, diag.as_ref())
            .unwrap();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 9,
                        "length": 6
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 9,
                        "length": 4
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 0,
                        "length": 0
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 9,
                        "length": 0
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "span": {
                        "offset": 9,
                        "length": 4
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 7,
                        "length": 4
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "x",
                    "span": {
                        "offset": 9,
                        "length": 4
                    }
                },
                {
                    "label": "y",
                    "span": {
                        "offset": 14,
                        "length": 4
                    }
                },
                {
                    "label": "z",
                    "span": {
                        "offset": 24,
                        "length": 4
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "these two lines",
                    "span": {
                        "offset": 9,
                        "length": 11
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "block 1",
                    "span": {
                        "offset": 0,
                        "length": 50
                    }
                },
                {
                    "label": "block 2",
                    "span": {
                        "offset": 10,
                        "length": 9
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "wtf?!\nit broke :(",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "block 1",
                    "span": {
                        "offset": 0,
                        "length": 50
                    }
                },
                {
                    "span": {
                        "offset": 10,
                        "length": 9
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 0,
                        "length": 10
                    }
                },
                {
                    "label": "also this bit",
                    "span": {
                        "offset": 20,
                        "length": 6
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
        assert_eq!(expected, out);
        Ok(())
    }

    #[test]
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 0,
                        "length": 8
                    }
                },
                {
                    "label": "also this bit",
                    "span": {
                        "offset": 9,
                        "length": 10
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
        assert_eq!(expected, out);
        Ok(())
    }

    #[test]
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
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 0,
                        "length": 8
                    }
                },
                {
                    "label": "also this bit",
                    "span": {
                        "offset": 10,
                        "length": 10
                    }
                }
            ],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
        assert_eq!(expected, out);
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
        let expected: String = r#"
        {
            "message": "oops!",
            "severity": "error",
            "url": "https://example.com",
            "help": "try doing it better next time?",
            "labels": [],
            "related": []
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
        assert_eq!(expected, out);
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
            related: vec![
                MyBad {
                    src: NamedSource::new("bad_file2.rs", src.clone()),
                    highlight: (0, 6).into(),
                    related: vec![],
                },
                MyBad {
                    src: NamedSource::new("bad_file3.rs", src),
                    highlight: (0, 6).into(),
                    related: vec![],
                },
            ],
        };
        let out = fmt_report(err.into());
        println!("Error: {}", out);
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 9,
                        "length": 4
                    }
                }
            ],
            "related": [{
                "message": "oops!",
                "code": "oops::my::bad",
                "severity": "error",
                "help": "try doing it better next time?",
                "filename": "bad_file2.rs",
                "labels": [
                    {
                        "label": "this bit here",
                        "span": {
                            "offset": 0,
                            "length": 6
                        }
                    }
                ],
                "related": []
            },{
                "message": "oops!",
                "code": "oops::my::bad",
                "severity": "error",
                "help": "try doing it better next time?",
                "filename": "bad_file3.rs",
                "labels": [
                    {
                        "label": "this bit here",
                        "span": {
                            "offset": 0,
                            "length": 6
                        }
                    }
                ],
                "related": []
            }]
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
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
        #[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
        struct InnerError {
            #[label("this bit here")]
            highlight: SourceSpan,
        }

        let src = "source\n  text\n    here".to_string();
        let err = MyBad {
            src: NamedSource::new("bad_file.rs", src.clone()),
            highlight: (9, 4).into(),
            related: vec![
                InnerError {
                    highlight: (0, 6).into(),
                },
                InnerError {
                    highlight: (0, 6).into(),
                },
            ],
        };
        let out = fmt_report(err.into());
        println!("Error: {}", out);
        let expected: String = r#"
        {
            "message": "oops!",
            "code": "oops::my::bad",
            "severity": "error",
            "help": "try doing it better next time?",
            "filename": "bad_file.rs",
            "labels": [
                {
                    "label": "this bit here",
                    "span": {
                        "offset": 9,
                        "length": 4
                    }
                }
            ],
            "related": [{
                "message": "oops!",
                "code": "oops::my::bad",
                "severity": "error",
                "help": "try doing it better next time?",
                "filename": "bad_file.rs",
                "labels": [
                    {
                        "label": "this bit here",
                        "span": {
                            "offset": 0,
                            "length": 6
                        }
                    }
                ],
                "related": []
            },{
                "message": "oops!",
                "code": "oops::my::bad",
                "severity": "error",
                "help": "try doing it better next time?",
                "filename": "bad_file.rs",
                "labels": [
                    {
                        "label": "this bit here",
                        "span": {
                            "offset": 0,
                            "length": 6
                        }
                    }
                ],
                "related": []
            }]
        }"#
        .lines()
        .into_iter()
        .map(|s| s.trim_matches(|c| c == ' ' || c == '\n'))
        .collect();
        assert_eq!(expected, out);
        Ok(())
    }
}
