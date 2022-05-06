use miette::Diagnostic;

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
