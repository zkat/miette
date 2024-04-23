use miette::{Diagnostic, MietteDiagnostic};

fn assert_diagnostic<T: Diagnostic>() {}

#[test]
fn test_ref() {
    assert_diagnostic::<&MietteDiagnostic>()
}

#[test]
fn test_box() {
    assert_diagnostic::<Box<MietteDiagnostic>>()
}
