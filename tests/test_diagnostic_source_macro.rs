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

// Compiletest this:
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
enum EnumError {
    #[error("Some Error text")]
    SyntaxErr(#[from] AnErr),
}

// Compiletest this:
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
enum AnotherEnumError {
    #[error("Some other Error Text")]
    SyntaxErr {
        #[from]
        #[source]
        #[diagnostic_source]
        source: AnErr,
    },
}
