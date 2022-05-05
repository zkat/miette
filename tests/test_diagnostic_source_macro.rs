use miette::Diagnostic;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("AnErr")]
struct AnErr;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("TestError")]
struct TestError {
    #[diagnostic_source]
    asdf_inner_foo: AnErr,
}

#[test]
fn test_diagnostic_source() {
    let error = TestError {
        asdf_inner_foo: AnErr,
    };
    assert!(error.diagnostic_source().is_some());
}
