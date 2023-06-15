use std::sync::Arc;

use miette::{miette, Diagnostic, Report};
use thiserror::Error;

#[test]
fn test_source() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("Bar")]
    struct Bar;

    #[derive(Debug, Diagnostic, Error)]
    #[error("Foo")]
    struct Foo {
        #[source]
        bar: Bar,
    }

    let e = miette!(Foo { bar: Bar });
    let mut chain = e.chain();

    assert_eq!("Foo", chain.next().unwrap().to_string());
    assert_eq!("Bar", chain.next().unwrap().to_string());
    assert!(chain.next().is_none());
}

#[test]
fn test_source_boxed() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("Bar")]
    struct Bar;

    #[derive(Debug, Diagnostic, Error)]
    #[error("Foo")]
    struct Foo {
        #[source]
        bar: Box<dyn Diagnostic + Send + Sync>,
    }

    let error = miette!(Foo { bar: Box::new(Bar) });

    let mut chain = error.chain();

    assert_eq!("Foo", chain.next().unwrap().to_string());
    assert_eq!("Bar", chain.next().unwrap().to_string());
    assert!(chain.next().is_none());
}

#[test]
fn test_source_arc() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("Bar")]
    struct Bar;

    #[derive(Debug, Diagnostic, Error)]
    #[error("Foo")]
    struct Foo {
        #[source]
        bar: Arc<dyn Diagnostic + Send + Sync>,
    }

    let error = miette!(Foo { bar: Arc::new(Bar) });

    let mut chain = error.chain();

    assert_eq!("Foo", chain.next().unwrap().to_string());
    assert_eq!("Bar", chain.next().unwrap().to_string());
    assert!(chain.next().is_none());
}
