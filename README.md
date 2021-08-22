you run miette? You run her code like the software? Oh. Oh! Error code for
coder! Error code for One Thousand Lines!

## About

`miette` is a diagnostic library for Rust. It includes a series of
traits/protocols that allow you to hook into its error reporting facilities,
and even write your own error reports! It lets you define error types that can
print out like this (or in any format you like!):

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

## Table of Contents <!-- omit in toc -->

- [About](#about)
- [Features](#features)
- [Installing](#installing)
- [Example](#example)
- [Usage](#usage)
  - [... in libraries](#-in-libraries)
  - [... in application code](#-in-application-code)
  - [... in `main()`](#-in-main)
  - [... snippets](#-snippets)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Features

- Generic [Diagnostic] protocol, compatible (and dependent on) `std::error::Error`.
- Unique error codes on every [Diagnostic].
- Super handy derive macro for defining diagnostic metadata.
- Lightweight [`anyhow`](https://docs.rs/anyhow)/[`eyre`](https://docs.rs/eyre)-style error wrapper type, [DiagnosticReport],
  which can be returned from `main`.
- Generic support for arbitrary [Source]s for snippet data, with default support for `String`s included.

The `miette` crate also comes bundles with a default [DiagnosticReportPrinter] with the following features:

- Fancy graphical [diagnostic output](#about), using ANSI/Unicode text
- single- and multi-line highlighting support
- Screen reader/braille support, gated on [`NO_COLOR`](http://no-color.org/), and other heuristics.
- Fully customizable graphical theming (or overriding the printers entirely).
- Cause chain printing

## Installing

Using [`cargo-edit`](https://crates.io/crates/cargo-edit):

```sh
$ cargo add miette
```

## Example

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
use miette::DiagnosticResult;
fn this_fails() -> DiagnosticResult<()> {
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
Now to get everything printed nicely, just return a DiagnosticResult<()>
and you're all set!

Note: You can swap out the default reporter for a custom one using `miette::set_reporter()`
*/
fn pretend_this_is_main() -> DiagnosticResult<()> {
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

## Usage

### ... in libraries

`miette` is _fully compatible_ with library usage. Consumers who don't know
about, or don't want, `miette` features can safely use its error types as
regular [std::error::Error].

We highly recommend using something like [`thiserror`](https://docs.rs/thiserror) to define unique error types and error wrappers for your library.

While `miette` integrates smoothly with `thiserror`, it is _not required_. If
you don't want to use the [Diagnostic] derive macro, you can implement the
trait directly, just like with `std::error::Error`.

```rust
// lib/error.rs
use thiserror::Error;
use miette::Diagnostic;

#[derive(Error, Diagnostic, Debug)]
pub enum MyLibError {
    #[error(transparent)]
    #[diagnostic(code(my_lib::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Oops it blew up")]
    #[diagnostic(code(my_lib::bad_code))]
    BadThingHappened,
}
```

Then, return this error type from all your fallible public APIs. It's a best
practice to wrap any "external" error types in your error `enum` instead of
using something like [eyre](https://docs.rs/eyre) in a library.

### ... in application code

Application code tends to work a little differently than libraries. You don't
always need or care to define dedicated error wrappers for errors coming from
external libraries and tools.

For this situation, `miette` includes two tools: [DiagnosticReport] and
[IntoDiagnostic]. They work in tandem to make it easy to convert regular
`std::error::Error`s into [Diagnostic]s. Additionally, there's a
[DiagnosticResult] type alias that you can use to be more terse:

```rust
// my_app/lib/my_internal_file.rs
use miette::{IntoDiagnostic, DiagnosticResult};
use semver::Version;

pub fn some_tool() -> DiagnosticResult<Version> {
    Ok("1.2.x".parse().into_diagnostic("my_app::semver::parse_error")?)
}
```

### ... in `main()`

`main()` is just like any other part of your application-internal code. Use
`DiagnosticResult` as your return value, and it will pretty-print your
diagnostics automatically.

```rust
use miette::{DiagnosticResult, IntoDiagnostic};
use semver::Version;

fn pretend_this_is_main() -> DiagnosticResult<()> {
    let version: Version = "1.2.x".parse().into_diagnostic("my_app::semver::parse_error")?;
    println!("{}", version);
    Ok(())
}
```

### ... snippets

Along with its general error handling and reporting features, `miette` also
includes facilities for adding error spans and annotations/highlights to your
output. This can be very useful when an error is syntax-related, but you can
even use it to print out sections of your own source code!

To achieve this, `miette` defines its own lightweight [SourceSpan] type. This
is a basic byte-offset and length into an associated [Source] and, along with
the latter, gives `miette` all the information it needs to pretty-print some
snippets!

The easiest way to define errors like this is to use the `derive(Diagnostic)`
macro:

```rust
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
#[error("oops")]
#[diagnostic(code(my_lib::random_error))]
pub struct MyErrorType {
    // The `Source` that miette will use.
    src: String,

    // A snippet that points to `src`, our `Source`. The filename can be
    // provided at the callsite.
    #[snippet(src, "This is the snippet")]
    snip: SourceSpan,

    // A highlight for the `snip` snippet we defined above. This will
    // underline/mark the specific code inside the larger snippet context.
    //
    // The label is provided using `SourceSpan`'s label.
    #[highlight(snip)]
    err_span: SourceSpan,
}
```

## Acknowledgements

`miette` was not developed in a void. It owes enormous credit to various other projects and their authors:

- [`anyhow`](http://crates.io/crates/anyhow) and
  [`color-eyre`](https://crates.io/crates/color-eyre): these two enormously
  influential error handling libraries have pushed forward the experience of
  application-level error handling and error reporting. `miette`'s
  `DiagnosticReport` type is an attempt at a very very rough version of their
  `Report` types.
- [`thiserror`](https://crates.io/crates/thiserror) for setting the standard
  for library-level error definitions, and for being the inspiration behind
  `miette`'s derive macro.
- `rustc` and [@estebank](https://github.com/estebank) for their state-of-the-art
  work in compiler diagnostics.
- [`ariadne`](https://crates.io/crates/ariadne) for pushing forward how
  _pretty_ these diagnostics can really look!

## License

`miette` is released to the Rust community under the [Apache license 2.0](./LICENSE).

It also includes some code taken from [`eyre`](https://github.com/yaahc/eyre),
and some from [`thiserror`](https://github.com/dtolnay/thiserror), also under
the Apache License. Some code is taken from
[`ariadne`](https://github.com/zesterer/ariadne), which is MIT licensed.
