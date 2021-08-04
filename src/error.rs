pub use std::io;

pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum MietteError {
    #[error(transparent)]
    IoError(#[from] io::Error),
}
