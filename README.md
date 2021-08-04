you FAIL miette? you fail her compiler like the unsafe C program? oh! oh! jail for mother! jail for mother for One Thousand Years!!!!

(I'm sorry, I'll come up with a better pun later.)

## Examples

Here's an example of using something like `thisdiagnostic` to define Diagnostics declaratively.

```rust
use thiserror::Error;
use thisdiagnostic::Diagnostic;

#[derive(Error, Diagnostic)]
pub enum MyDiagnostic {
    /// Generally happens because you did some specific thing wrong. The
    /// reason is actually something like <...>
    #[diagnostic(
        code = fatal::error, // Translates to `mycrate::fatal::error` and creates
                             // a crate-wide alias so you can just search for that
                             // exact full string in rustdoc.
        help = "Please consider doing things differently next time.",
        backtrace = true, // Enable showing a backtrace when this is printed.
    )]
    #[error("Oopsie poopsie it all exploded.")]
    FatalError,

    /// This one usually resolves on its own, don't worry about it.
    #[diagnostic(
        code = nice::warning,
        help = "This is mostly to help you!",
        severity = warning
    )]
    #[error("This might break in the future.")]
    NiceWarning,

    /// This diagnostic includes code spans!
    #[diagnostic(
        code = math::bad_arithmetic,
        help = "Convert {bad_var} into a {good_type} and try again."
    )]
    #[error("Tried to add a {bad_type} to a {good_type}")]
    BadArithmetic {
        // Regular error metadata for programmatic use.
        good_type: Type,
        bad_type: Type,
        bad_var: Var,

        // Anything implementing the Source trait can be used as a source.
        src: PathBuf,
        other_src: String,

        // The context is the span of code that will be rendered.
        // There can be multiple contexts in a Diagnostic.
        #[context(src, "This region is where things went wrong.")]
        ctx: SourceSpan,

        // Highlights underline and label specific subsections of the context.
        #[highlight(ctx, "This is a {bad_type}")]
        bad_var_span: SourceSpan, // These can span multiple lines!

        // They can be optional!
        #[highlight(ctx, "This is a {good_type}")]
        good_var_span: Option<SourceSpan>,
    },

    #[error(transparent)]
    #[diagnostic(transparent)]
    // renders as "some_library::<code_subpath>". No docs needed.
    // You must re-export `SomeLibraryError` from your crate if you want
    // users to be able to find its error codes on your own rustdoc search box.
    SomeLibraryError(#[from] some_library::SomeLibraryError)
}
```
