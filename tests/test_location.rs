use std::panic::Location;

use miette::{Diagnostic, IntoDiagnostic, WrapErr};

struct LocationHandler {
    actual: Option<&'static str>,
    expected: &'static str,
}

impl LocationHandler {
    fn new(expected: &'static str) -> Self {
        LocationHandler {
            actual: None,
            expected,
        }
    }
}

impl miette::ReportHandler for LocationHandler {
    fn debug(
        &self,
        _error: &(dyn Diagnostic + 'static),
        _f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        // we assume that if the compiler is new enough to support
        // `track_caller` that we will always have `actual` be `Some`, so we can
        // safely skip the assertion if the location is `None` which should only
        // happen in older rust versions.
        if let Some(actual) = self.actual {
            assert_eq!(self.expected, actual);
        }

        Ok(())
    }

    fn track_caller(&mut self, location: &'static Location<'static>) {
        dbg!(location);
        self.actual = Some(location.file());
    }
}

#[test]
fn test_wrap_err() {
    let _ = miette::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .into_diagnostic()
        .wrap_err("oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_wrap_err_with() {
    let _ = miette::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .into_diagnostic()
        .wrap_err_with(|| "oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_context() {
    let _ = miette::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .into_diagnostic()
        .context("oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_with_context() {
    let _ = miette::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .into_diagnostic()
        .with_context(|| "oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}
