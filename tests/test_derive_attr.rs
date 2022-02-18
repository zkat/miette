// Testing of the `diagnostic` attr used by derive(Diagnostic)
use miette::{Diagnostic, NamedSource, SourceSpan};
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
        }
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
        }
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
}

#[test]
fn enum_prefers_variant_attr() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    enum MyBad {
        #[diagnostic(code(error::on::variant))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        }
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
}

#[test]
fn enum_combines_base_variant() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(help("try doing it correctly"))]
    enum MyBad {
        #[diagnostic(code(error::on::variant))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        }
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
    assert_eq!(
        err.help().unwrap().to_string(),
        "try doing it correctly"
    );
}

#[test]
fn enum_combines_all_attrs() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    #[diagnostic(code(error::on::base))]
    #[diagnostic(help("try doing it correctly"))]
    #[diagnostic(url("https://example.com/foo/bar"))]
    enum MyBad {
        #[diagnostic(code(error::on::variant))]
        Only {
            #[source_code]
            src: NamedSource,
            #[label("this bit here")]
            highlight: SourceSpan,
        }
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: NamedSource::new("bad_file.rs", src),
        highlight: (9, 4).into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "error::on::variant");
    assert_eq!(
        err.help().unwrap().to_string(),
        "try doing it correctly"
    );
    assert_eq!(
        err.url().unwrap().to_string(),
        "https://example.com/foo/bar".to_string()
    );
}
