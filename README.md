# miette

you run miette? You run her code like the software? Oh. Oh! Error code for
coder! Error code for One Thousand Lines!

## About

`miette` is a diagnostic definition library for Rust. It includes a series of
protocols that allow you to hook into its error reporting facilities, and even
write your own error reports! It lets you define error types that can print out
like this (or in any format you like!):

```sh
Error: Error[oops::my::bad]: oops it broke!

[bad_file.rs] This is the part that broke:

    1  | source
    2  |   text
    ⫶  |   ^^^^ this bit here
    3  |     here

﹦try doing it better next time?
```

The [Diagnostic] trait in `miette` is an extension of `std::error::Error` that
adds various facilities like [Severity], error codes that could be looked up
by users, and snippet display with support for multiline reports, arbitrary
[Source]s, and pretty printing.

While the `miette` crate bundles some baseline implementations for [Source]
and [DiagnosticReporter], it's intended to define a protocol that other crates
can build on top of to provide rich error reporting, and encourage an
ecosystem that leans on this extra metadata to provide it for others in a way
that's compatible with [std::error::Error]

## Installing

Using [`cargo-edit`](https://crates.io/crates/cargo-edit):

```sh
$ cargo add miette
```

## Example and Guide

```rust
/*
First, you implement a regular `std::error::Error` type.

`thiserror` is a great way to do so, and plays extremely nicely with `miette`!
*/

use thiserror::Error;

#[derive(Error)]
#[error("oops it broke!")]
struct MyBad {
    snippets: Vec<DiagnosticSnippet>,
}

/*
Next, we have to implement the `Diagnostic` trait for it:
*/

use miette::{Diagnostic, Severity, DiagnosticSnippet};

impl Diagnostic for MyBad {
    fn code(&self) -> &(dyn std::fmt::Display + 'static) {
        &"oops::my::bad"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn help(&self) -> Option<&[&str]> {
        Some(&["try doing it better next time?"])
    }

    fn snippets(&self) -> Option<&[DiagnosticSnippet]> {
        Some(&self.snippets)
    }
}

/*
Then, we implement `std::fmt::Debug` using the included `MietteReporter`,
which is able to pretty print diagnostics reasonably well.

You can use any reporter you want here, or no reporter at all,
but `Debug` is required by `std::error::Error`, so you need to at
least derive it.

Make sure you pull in the `miette::DiagnosticReporter` trait!.
*/
use std::fmt;

use miette::{DiagnosticReporter, MietteReporter};

impl fmt::Debug for MyBad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        MietteReporter.debug(self, f)
    }
}

/*
Now we can use `miette`~
*/
use miette::{MietteError, SourceSpan};

fn make_my_error() -> MyBad {
    // You can use plain strings as a `Source`, bu the protocol is fully extensible!
    let src = "source\n  text\n    here".to_string();

    // The Rust runtime will use `{:?}` (Debug) to print any error you return
    // from `main`!
    MyBad {
        // Snippets are **fully optional**, but in some use cases can provide
        // additional contextual detail for users!
        //
        // This is all you need to write to get `rustc`-style, rich error reports!
        //
        // See the docs for `DiagnosticSnippet` to learn more about how to
        // construct these objects!
        snippets: vec![DiagnosticSnippet {
            message: Some("This is the part that broke".into()),
            source_name: "bad_file.rs".into(),
            source: Box::new(src.clone()),
            context: SourceSpan {
                start: 0.into(),
                end: (src.len() - 1).into(),
            },
            highlights: Some(vec![
                ("this bit here".into(), SourceSpan {
                    start: 9.into(),
                    end: 12.into(),
                })
            ]),
        }],
    }
}
```

And this is the output you'll get if you run this program:

```sh
Error: Error[oops::my::bad]: oops it broke!

[bad_file.rs] This is the part that broke:

    1  | source
    2  |   text
    ⫶  |   ^^^^ this bit here
    3  |     here

﹦try doing it better next time?
```

## License

`miette` is released to the Rust community under the [MIT license](./LICENSE).

It also includes some code taken from [`eyre`](https://github.com/yaahc/eyre),
also [under the MIT license](https://github.com/yaahc/eyre#license).
