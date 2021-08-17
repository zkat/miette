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
You can derive a Diagnostic from any `std::error::Error` type.

`thiserror` is a great way to define them so, and plays extremely nicely with `miette`!
*/
use std::sync::Arc;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic)]
#[error("oops it broke!")]
#[diagnostic(
    code(oops::my::bad),
    severity(Warning),
    help("try doing it better next time?"),
)]
struct MyBad {
    src: Arc<String>,
    filename: String,
    // Snippets and highlights can be included in the diagnostic!
    #[snippet(src, filename, "This is the part that broke")]
    snip: SourceSpan,
    #[highlight(snip, "this bit here")]
    bad_bit: SourceSpan,
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

fn pretend_this_is_main() -> Result<(), MyBad> {
    // You can use plain strings as a `Source`, bu the protocol is fully extensible!
    let src = "source\n  text\n    here".to_string();
    let len = src.len();

    Err(MyBad {
        src: Arc::new(src),
        filename: "bad_file.rs".into(),
        snip: (0, len).into(),
        bad_bit: (9, 3).into(),
    })
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
