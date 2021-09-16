#![doc = include_str!("../README.md")]
#![deny(missing_docs, missing_debug_implementations, nonstandard_style)]
#![warn(unreachable_pub, rust_2018_idioms)]

pub use miette_derive::*;

pub use error::*;
pub use eyreish::*;
#[cfg(feature = "fancy")]
pub use handler::*;
pub use handlers::*;
pub use named_source::*;
pub use protocol::*;

mod chain;
mod error;
mod eyreish;
#[cfg(feature = "fancy")]
mod handler;
mod handlers;
mod named_source;
mod protocol;
mod source_impls;
