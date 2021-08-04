pub use std::io;

pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum MietteError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("The given offset is outside the bounds of its Source")]
    OutOfBounds
}
