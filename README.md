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

`miette` also includes a (lightweight) `anyhow`/`eyre`-style
[DiagnosticReport] type which can be returned from application-internal
functions to make the `?` experience nicer. It's extra easy to use when using
[DiagnosticResult]!

While the `miette` crate bundles some baseline implementations for [Source]
and [DiagnosticReportPrinter], it's intended to define a protocol that other crates
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

`thiserror` is a great way to define them, and plays nicely with `miette`!
*/
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("oops it broke!")]
#[diagnostic(
    code(oops::my::bad),
    severity(Warning),
    help("try doing it better next time?"),
)]
struct MyBad {
    // The Source that we're gonna be printing snippets out of.
    src: String,
    // Snippets and highlights can be included in the diagnostic!
    #[snippet(src, "This is the part that broke")]
    snip: SourceSpan,
    #[highlight(snip)]
    bad_bit: SourceSpan,
}

/*
Now let's define a function!

Use this DiagnosticResult type (or its expanded version) as the return type
throughout your app (but NOT your libraries! Those should always return concrete
types!).
*/
use miette::DiagnosticResult as Result;
fn this_fails() -> Result<()> {
    // You can use plain strings as a `Source`, or anything that implements
    // the one-method `Source` trait.
    let src = "source\n  text\n    here".to_string();
    let len = src.len();

    Err(MyBad {
        src,
        snip: ("bad_file.rs", 0, len).into(),
        bad_bit: ("this bit here", 9, 4).into(),
    })?;

    Ok(())
}

/*
Now to get everything printed nicely, just return a Result<(), DiagnosticReport>
and you're all set!

Note: You can swap out the default reporter for a custom one using `miette::set_reporter()`
*/
fn pretend_this_is_main() -> Result<()> {
    // kaboom~
    this_fails()?;

    Ok(())
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

`miette` is released to the Rust community under the [Apache license 2.0](./LICENSE).

It also includes some code taken from [`eyre`](https://github.com/yaahc/eyre),
and some from [`thiserror`](https://github.com/dtolnay/thiserror), also under
the Apache License. Some code is taken from
[`ariadne`](https://github.com/zesterer/ariadne), which is MIT licensed.
