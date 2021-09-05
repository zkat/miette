#![doc = include_str!("../README.md")]
#![deny(missing_docs, missing_debug_implementations, nonstandard_style)]
#![warn(unreachable_pub, rust_2018_idioms)]

pub use miette_derive::*;

pub use error::*;
pub use eyreish::*;
pub use handlers::*;
pub use named_source::*;
pub use protocol::*;

mod chain;
mod error;
mod eyreish;
mod handlers;
mod named_source;
mod protocol;
mod source_impls;

#[cfg(doctest)]
mod compile_test;
