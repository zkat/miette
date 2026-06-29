/*!
Default trait implementations for [`Diagnostic`].
*/
extern crate alloc;

use alloc::boxed::Box;
use core::{convert::Infallible, fmt::Display};

use crate::{Diagnostic, LabeledSpan, Severity, SourceCode};

// Note: Infallible implements core::error::Error since Rust 1.81,
// so we don't need to provide our own StdError impl.

impl Diagnostic for Infallible {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match *self {}
    }

    fn severity(&self) -> Option<Severity> {
        match *self {}
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match *self {}
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match *self {}
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match *self {}
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match *self {}
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        match *self {}
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match *self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Report;

    /// Test that [`Infallible`] implements [`Diagnostic`] by seeing if a function that's generic over `Diagnostic`
    /// will accept `Infallible` as a type parameter.
    #[test]
    fn infallible() {
        let _ = Report::new::<Infallible>;
    }
}
