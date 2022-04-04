// Testing of the `diagnostic` attr used by derive(Diagnostic)
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;

#[test]
fn enum_uses_base_attr() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    enum MyBad {
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::base");
}

#[test]
fn enum_uses_variant_attr() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    enum MyBad {
        #[diagnostic(code(error::on::variant))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
}

#[test]
fn multiple_attrs_allowed_on_item() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    #[diagnostic(help("try doing it correctly"))]
    enum MyBad {
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::base");
    assert_eq!(err.help().unwrap().to_string(), "try doing it correctly");
}

#[test]
fn multiple_attrs_allowed_on_variant() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    enum MyBad {
        #[diagnostic(code(error::on::variant))]
        #[diagnostic(help("try doing it correctly"))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
    assert_eq!(err.help().unwrap().to_string(), "try doing it correctly");
}

#[test]
fn attrs_can_be_split_between_item_and_variants() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    enum MyBad {
        #[diagnostic(help("try doing it correctly"))]
        #[diagnostic(url("https://example.com/foo/bar"))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::base");
    assert_eq!(err.help().unwrap().to_string(), "try doing it correctly");
    assert_eq!(
        err.url().unwrap().to_string(),
        "https://example.com/foo/bar".to_string()
    );
}

#[test]
fn attr_not_required() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    enum MyBad {
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    let err_span = err.labels().unwrap().next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
}
