/*!
Default trait implementations for [`Diagnostic`].
*/

use std::{convert::Infallible, fmt::Display, hint::unreachable_unchecked};

use crate::{Diagnostic, LabeledSpan, Severity, SourceCode};

// Since all trait methods for [`Diagnostic`] take a reference to `self`, there must be an instance of `Self`.
// However, since an instance of [`Infallible`] can never be constructed, these methods can never be called.
// Therefore, these methods are unreachable, and can be safely optimized away by the compiler.
#[allow(clippy::undocumented_unsafe_blocks)]
impl Diagnostic for Infallible {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        unsafe { unreachable_unchecked() }
    }

    fn severity(&self) -> Option<Severity> {
        unsafe { unreachable_unchecked() }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        unsafe { unreachable_unchecked() }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        unsafe { unreachable_unchecked() }
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        unsafe { unreachable_unchecked() }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        unsafe { unreachable_unchecked() }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        unsafe { unreachable_unchecked() }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        unsafe { unreachable_unchecked() }
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
