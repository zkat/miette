use miette::{Diagnostic, TypedReport};
use thiserror::Error;

#[test]
fn into_typed() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    struct MyBad {
        #[source_code]
        src: String,
        #[label("this bit here")]
        highlight: (usize, usize),
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src,
        highlight: (9, 4),
    };
    let typed_err: TypedReport<_> = err.into();
    assert_eq!(typed_err.code().unwrap().to_string(), "error::on::base");
}

#[test]
fn backtrace_retention() {
    #[derive(Debug, Error)]
    #[error("oops!")]
    struct MyBad;

    #[derive(Debug, Error)]
    #[error("also fail: {0}")]
    struct AlsoBad(#[from] MyBad);

    let typed_err: TypedReport<_> = MyBad.into();
    let backtrace1 = typed_err.backtrace().to_string();

    let other: TypedReport<AlsoBad> = typed_err.into();

    let backtrace2 = other.backtrace().to_string();

    assert_eq!(backtrace1, backtrace2);
}

