use std::{fmt, io};

use thiserror::Error;

use crate::Diagnostic;

/**
Error enum for miette. Used by certain operations in the protocol.
*/
#[derive(Debug, Error)]
pub enum MietteError {
    /// Wrapper around [`std::io::Error`]. This is returned when something went
    /// wrong while reading a [`SourceCode`](crate::SourceCode).
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// Returned when a [`SourceSpan`](crate::SourceSpan) extends beyond the
    /// bounds of a given [`SourceCode`](crate::SourceCode).
    #[error("The given offset is outside the bounds of its Source")]
    OutOfBounds,
}

impl Diagnostic for MietteError {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        match self {
            MietteError::IoError(_) => Some(Box::new("miette::io_error")),
            MietteError::OutOfBounds => Some(Box::new("miette::span_out_of_bounds")),
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        match self {
            MietteError::IoError(_) => None,
            MietteError::OutOfBounds => Some(Box::new(
                "Double-check your spans. Do you have an off-by-one error?",
            )),
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        let crate_version = env!("CARGO_PKG_VERSION");
        Some(Box::new(format!(
            "https://docs.rs/miette/{crate_version}/miette/enum.MietteError.html"
        )))
    }
}
