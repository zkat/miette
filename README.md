you run miette? You run her code like the software? Oh. Oh! Error code for
coder! Error code for One Thousand Lines!

## About

`miette` is a diagnostic library for Rust. It includes a series of protocols
that allow you to hook into its error reporting facilities, and even write
your own error reports! It lets you define error types that can print out like
this (or in any format you like!):

<img src="https://raw.githubusercontent.com/zkat/miette/main/images/serde_json.png" alt="Hi! miette also includes a screen-reader-oriented diagnostic printer that's enabled in various situations, such as when you use NO_COLOR or CLICOLOR settings, or on CI. This behavior is also fully configurable and customizable. For example, this is what this particular diagnostic will look like when the narrated printer is enabled:
\
Error: Received some bad JSON from the source. Unable to parse.
    Caused by: missing field `foo` at line 1 column 1700
\
Begin snippet for https://api.nuget.org/v3/registration5-gz-semver2/json.net/index.json starting
at line 1, column 1659
\
snippet line 1: gs&quot;:[&quot;json&quot;],&quot;title&quot;:&quot;&quot;,&quot;version&quot;:&quot;1.0.0&quot;},&quot;packageContent&quot;:&quot;https://api.nuget.o
    highlight starting at line 1, column 1699: last parsing location
\
diagnostic help: This is a bug. It might be in ruget, or it might be in the source you're using,
but it's definitely a bug and should be reported.
diagnostic error code: ruget::api::bad_json
" />

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
that's compatible with [std::error::Error].

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
#[error("oops!")]
#[diagnostic(
    code(oops::my::bad),
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

<img src="https://raw.githubusercontent.com/zkat/miette/main/images/single-line-example.png" alt="
Narratable printout:
\
Error: oops!
    Diagnostic severity: error
\
Begin snippet for bad_file.rs starting at line 1, column 1
\
snippet line 1: source
snippet line 2:   text
    highlight starting at line 2, column 3: these two lines
snippet line 3:     here
\
diagnostic help: try doing it better next time?
diagnostic error code: oops::my::bad
">

## License

`miette` is released to the Rust community under the [Apache license 2.0](./LICENSE).

It also includes some code taken from [`eyre`](https://github.com/yaahc/eyre),
and some from [`thiserror`](https://github.com/dtolnay/thiserror), also under
the Apache License. Some code is taken from
[`ariadne`](https://github.com/zesterer/ariadne), which is MIT licensed.
