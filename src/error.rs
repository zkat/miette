use std::io;

use thiserror::Error;

/**
Error enum for miette. Used by certain operations in the protocol.
*/
#[derive(Debug, Error)]
pub enum MietteError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("The given offset is outside the bounds of its Source")]
    OutOfBounds
}
