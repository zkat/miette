use miette::{miette, Diagnostic, Report};
use std::error::Error as StdError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("outer")]
struct MyError {
    source: io::Error,
}
impl Diagnostic for MyError {}

#[test]
fn test_boxed_str_diagnostic() {
    let error = Box::<dyn Diagnostic + Send + Sync>::from("oh no!");
    let error: Report = miette!(error);
    assert_eq!("oh no!", error.to_string());
    assert_eq!(
        "oh no!",
        error
            .downcast_ref::<Box<dyn Diagnostic + Send + Sync>>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_boxed_str_stderr() {
    let error = Box::<dyn StdError + Send + Sync>::from("oh no!");
    let error: Report = miette!(error);
    assert_eq!("oh no!", error.to_string());
    assert_eq!(
        "oh no!",
        error
            .downcast_ref::<Box<dyn StdError + Send + Sync>>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_boxed_thiserror() {
    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!"),
    };
    let error: Report = miette!(error);
    assert_eq!("oh no!", error.source().unwrap().to_string());
}

#[test]
fn test_boxed_miette() {
    let error: Report = miette!("oh no!").wrap_err("it failed");
    let error = miette!(error);
    assert_eq!("oh no!", error.source().unwrap().to_string());
}

#[test]
#[ignore = "I don't know why this isn't working but it needs fixing."]
fn test_boxed_sources() {
    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!"),
    };
    let error = Box::<dyn Diagnostic + Send + Sync>::from(error);
    let error: Report = miette!(error).wrap_err("it failed");
    assert_eq!("it failed", error.to_string());
    assert_eq!("outer", error.source().unwrap().to_string());
    assert_eq!(
        "oh no!",
        error
            .source()
            .expect("outer")
            .source()
            .expect("inner")
            .to_string()
    );
}
