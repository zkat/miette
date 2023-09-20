use miette::Diagnostic;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("A complex error happened")]
struct SourceError {
    #[source_code]
    code: String,
    #[help]
    help: String,
    #[label("here")]
    label: (usize, usize),
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("AnErr")]
struct AnErr;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
struct TestStructError {
    #[diagnostic_source]
    asdf_inner_foo: SourceError,
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
enum TestEnumError {
    Without,
    WithTuple(#[diagnostic_source] AnErr),
    WithStruct {
        #[diagnostic_source]
        inner: AnErr,
    },
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
struct TestTupleError(#[diagnostic_source] AnErr);

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
struct TestBoxedError(#[diagnostic_source] Box<dyn Diagnostic>);

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
struct TestArcedError(#[diagnostic_source] std::sync::Arc<dyn Diagnostic>);

#[test]
fn test_diagnostic_source() {
    let error = TestStructError {
        asdf_inner_foo: SourceError {
            code: String::new(),
            help: String::new(),
            label: (0, 0),
        },
    };
    assert!(error.diagnostic_source().is_some());

    let error = TestEnumError::Without;
    assert!(error.diagnostic_source().is_none());

    let error = TestEnumError::WithTuple(AnErr);
    assert!(error.diagnostic_source().is_some());

    let error = TestEnumError::WithStruct { inner: AnErr };
    assert!(error.diagnostic_source().is_some());

    let error = TestTupleError(AnErr);
    assert!(error.diagnostic_source().is_some());

    let error = TestBoxedError(Box::new(AnErr));
    assert!(error.diagnostic_source().is_some());

    let error = TestArcedError(std::sync::Arc::new(AnErr));
    assert!(error.diagnostic_source().is_some());
}

#[test]
fn test_diagnostic_source_pass_extra_info() {
    let diag = TestBoxedError(Box::new(SourceError {
        code: String::from("Hello\nWorld!"),
        help: String::from("Have you tried turning it on and off again?"),
        label: (1, 4),
    }));
    let mut out = String::new();
    miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor())
        .with_width(80)
        .with_footer("this is a footer".into())
        .render_report(&mut out, &diag)
        .unwrap();
    println!("Error: {}", out);
    let expected = r#"  × TestError
  ╰─▶   × A complex error happened
         ╭─[1:2]
       1 │ Hello
         ·  ──┬─
         ·    ╰── here
       2 │ World!
         ╰────
        help: Have you tried turning it on and off again?
      

  this is a footer
"#
    .to_string();
    assert_eq!(expected, out);
}

#[test]
fn test_diagnostic_source_is_output() {
    let diag = TestStructError {
        asdf_inner_foo: SourceError {
            code: String::from("right here"),
            help: String::from("That's where the error is!"),
            label: (6, 4),
        },
    };
    let mut out = String::new();
    miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor())
        .with_width(80)
        .render_report(&mut out, &diag)
        .unwrap();
    println!("{}", out);

    let expected = r#"  × TestError
  ╰─▶   × A complex error happened
         ╭────
       1 │ right here
         ·       ──┬─
         ·         ╰── here
         ╰────
        help: That's where the error is!
      
"#;

    assert_eq!(expected, out);
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("A nested error happened")]
struct NestedError {
    #[source_code]
    code: String,
    #[label("here")]
    label: (usize, usize),
    #[diagnostic_source]
    the_other_err: Box<dyn Diagnostic>,
}

#[test]
fn test_nested_diagnostic_source_is_output() {
    let inner_error = TestStructError {
        asdf_inner_foo: SourceError {
            code: String::from("This is another error"),
            help: String::from("You should fix this"),
            label: (3, 4),
        },
    };
    let diag = NestedError {
        code: String::from("right here"),
        label: (6, 4),
        the_other_err: Box::new(inner_error),
    };
    let mut out = String::new();
    miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor())
        .with_width(80)
        .with_footer("Yooo, a footer".to_string())
        .render_report(&mut out, &diag)
        .unwrap();
    println!("{}", out);

    let expected = r#"  × A nested error happened
  ├─▶   × TestError
  │   
  ╰─▶   × A complex error happened
         ╭────
       1 │ This is another error
         ·    ──┬─
         ·      ╰── here
         ╰────
        help: You should fix this
      
   ╭────
 1 │ right here
   ·       ──┬─
   ·         ╰── here
   ╰────

  Yooo, a footer
"#;

    assert_eq!(expected, out);
}
