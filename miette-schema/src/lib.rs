#![deny(missing_docs)]

//! # miette-schema
//!
//! This library provides type definitions and derives for
//! Serializing/Deserializing miette's JSON output using serde, and for getting
//! a schema of the format. The schema is based on [schemars][], which
//! understands your serde attributes.
//!
//! This idea is that if you have your own JSON output that may include miette
//! JSON output, you can slap the top-level [`Diagnostic`][] type in your own
//! types and get serde able to read/write it properly, and if you want a schema
//! for your entire output format then you can just slap `#[derive(JsonSchema)]`
//! on your own types and use `schemars::schema_for!(MyType)` to get the final
//! JSON Schema.
//!
//! The type definitions here are more conservative than what miette's JSON
//! output actually looks like to make the schema maximally
//! forward/backward-compatible with past/future versions of miette.
//!
//! Basically you should ideally assume every field is Optional because
//! older versions might be missing it and newer versions might remove it. Not
//! because we have any plans to break the format, but just because it's Best
//! Practice to prepare for that eventuality and interchange formats inevitably
//! result in version drift. We implement this by placing `#[serde(default)]` on
//! everything we can.
//!
//! This pretty closely matches how miette's JSON Output works in practice
//! today: every field is always emitted, and it just emits an empty string or
//! an empty array when there's no value. So in practice the
//! `#[serde(default)]`s won't ever be stressed, because miette itself pre-bakes
//! them in. So write code that's aware of empty strings!
//!
//! You also need to be tolerant of additional things being added in the future,
//! but thankfully that's just how Json Schema and serde already work, by
//! silently dropping unknown values on the floor... except for enum variants.
//!
//! We had to add an extra Unknown variant to every enum to capture "unknown
//! variants". Simply making the enum `#[non_exhaustive]` is insufficient
//! because the Rust compiler needs to put *something* there at runtime.
//!
//! [schemars]: https://docs.rs/schemars/latest/schemars/

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A miette diagnostic, the top level type of miette's json output
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct Diagnostic {
    /// The primary error message
    #[serde(default)]
    message: String,
    /// The error code
    ///
    /// e.g. "oops::my::bad" or "E1312"
    #[serde(default)]
    code: String,
    /// The severity of the error (error, warning, advice, ...)
    #[serde(default)]
    severity: Severity,
    /// The underlying causes of this error (similar to a backtrace)
    #[serde(default)]
    causes: Vec<String>,
    /// A URL to visit to get more information on this error
    #[serde(default)]
    url: String,
    /// An additional piece of advice on how to address the diagnostic
    ///
    /// e.g. "try removing this trailing comma"
    #[serde(default)]
    help: String,
    /// Labels/spans referring to the locations in the source that are relevant
    /// to the diagnostic
    ///
    /// See "filename" for the source file these labels refer to
    ///
    /// e.g. "here's the extra comma you should remove" (offset: 100, len: 1)
    #[serde(default)]
    labels: Vec<Label>,
    /// Related Diagnostics nested under this one
    ///
    /// This can be used to report multiple diagnostics at once, by e.g.
    /// having a vague top-level diagnostic like "failed to compile" with the
    /// more specific issues nested underneath it.
    #[serde(default)]
    related: Vec<Diagnostic>,
    /// The name of the source file that caused the diagnostic
    #[serde(default)]
    filename: String,
}

/// The severity of a diagnostic
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum Severity {
    /// This is an error
    #[serde(rename = "error")]
    #[default]
    Error,
    /// This is a warning
    #[serde(rename = "warning")]
    Warning,
    /// This is just some advice
    #[serde(rename = "advice")]
    Advice,
    /// A dummy variant for forward/backward-compatibility with other versions
    /// of miette which may one day introduce more kinds of Severity. Any
    /// unknown ones will be mapped to this variant.
    #[serde(other, rename = "_unknown")]
    Unknown,
}

/// A label/span indicating relevant portions of a source file for a Diagnostic
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct Label {
    /// The label/message for the span
    #[serde(default)]
    label: String,
    /// The actual span/range of source code that we're referring to
    #[serde(default)]
    span: Span,
}

/// A span/range of source code
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct Span {
    /// The byte offset where the span starts
    #[serde(default)]
    offset: u64,
    /// How many bytes the span contains
    #[serde(default)]
    length: u64,
}

impl Diagnostic {
    /// Get the JSON Schema for a Diagnostic
    pub fn json_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Diagnostic)
    }
}

#[test]
fn emit() {
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;
    use std::path::PathBuf;

    let schema = Diagnostic::json_schema();
    let json_schema = serde_json::to_string_pretty(&schema).unwrap();

    // FIXME: (?) we should use something like xtask to update the schema, but this
    // works ok.
    let root = std::env!("CARGO_MANIFEST_DIR");
    let schema = PathBuf::from(root).join("miette-json-schema.json");
    let file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(schema)
        .unwrap();
    let mut file = BufWriter::new(file);
    writeln!(&mut file, "{json_schema}").unwrap();
}
