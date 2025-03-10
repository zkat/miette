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
            src: NamedSource<String>,
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
            src: NamedSource<String>,
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
            src: NamedSource<String>,
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
            src: NamedSource<String>,
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
            src: NamedSource<String>,
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
            src: NamedSource<String>,
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

fn assert_impl_diagnostic<T: Diagnostic>() {}

#[test]
fn transparent_generic() {
    #[derive(Debug, Diagnostic, Error)]
    enum Combined<T> {
        #[error(transparent)]
        #[diagnostic(transparent)]
        Other(T),
        #[error("foo")]
        Custom,
    }

    std::hint::black_box(Combined::<i32>::Other(1));
    std::hint::black_box(Combined::<i32>::Custom);

    assert_impl_diagnostic::<Combined<miette::MietteDiagnostic>>();
}

#[test]
fn generic_label() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[label]
        label: T,
    }

    assert_impl_diagnostic::<Combined<SourceSpan>>();
    assert_impl_diagnostic::<Combined<(usize, usize)>>();
}

#[test]
fn generic_source_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[source_code]
        label: T,
    }

    assert_impl_diagnostic::<Combined<String>>();
}

#[test]
fn generic_optional_source_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[source_code]
        label: Option<T>,
    }

    assert_impl_diagnostic::<Combined<String>>();
}

#[test]
fn generic_label_primary() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[label(primary)]
        label: T,
    }

    assert_impl_diagnostic::<Combined<SourceSpan>>();
    assert_impl_diagnostic::<Combined<(usize, usize)>>();
}

#[test]
fn generic_label_collection() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[label(collection)]
        label: Vec<T>,
    }

    assert_impl_diagnostic::<Combined<SourceSpan>>();
    assert_impl_diagnostic::<Combined<(usize, usize)>>();
}

#[test]
fn generic_label_generic_collection() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[label(collection)]
        label: T,
    }

    assert_impl_diagnostic::<Combined<Vec<SourceSpan>>>();
    assert_impl_diagnostic::<Combined<Vec<(usize, usize)>>>();
}

#[test]
fn generic_related() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("foo")]
    struct Combined<T> {
        #[related]
        label: Vec<T>,
    }

    assert_impl_diagnostic::<Combined<miette::MietteDiagnostic>>();
}

#[test]
fn generic_diagnostic_source() {
    #[derive(Debug, Diagnostic, Error)]
    enum Combined<T> {
        #[error(transparent)]
        Other(#[diagnostic_source] T),
        #[error("foo")]
        Custom,
    }

    std::hint::black_box(Combined::<i32>::Other(1));
    std::hint::black_box(Combined::<i32>::Custom);

    assert_impl_diagnostic::<Combined<miette::MietteDiagnostic>>();
}

#[test]
fn generic_not_influencing_default() {
    #[derive(Debug, Diagnostic, Error)]
    enum Combined<T> {
        #[error("bar")]
        Other(T),
        #[error("foo")]
        Custom,
    }

    std::hint::black_box(Combined::<i32>::Other(1));
    std::hint::black_box(Combined::<i32>::Custom);

    assert_impl_diagnostic::<Combined<i32>>();
}
