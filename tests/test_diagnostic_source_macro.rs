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
    asdf_inner_foo: AnErr,
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
        asdf_inner_foo: AnErr,
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
        help: format!("Have you tried turning it on and off again?"),
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
         ╭─[1:1]
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
