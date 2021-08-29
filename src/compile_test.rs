//! A hacky but perfectly good method of adding compile_fail doctests. You can't do this in a
//! regular tests/blah.rs file.

/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
///
/// #[derive(Debug, Diagnostic, Error)]
/// enum Variants {
///     #[error("no")]
///     #[diagnostic(transparent)]
///     One,
/// }
/// ```
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
///
/// #[derive(Debug, Diagnostic, Error)]
/// enum Variants {
///     #[error("no")]
///     #[diagnostic(transparent)]
///     One {
///         one: Foo,
///         two: u32,
///     },
/// }
/// ```
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
///
/// #[derive(Debug, Diagnostic, Error)]
/// enum Variants {
///     #[error("no")]
///     #[diagnostic(transparent)]
///     One(Foo, u32),
/// }
/// ```
///
#[allow(dead_code)]
#[doc(hidden)]
struct SingleFieldTests;

/// Directly on a struct with any other arg
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(transparent, code(invalid::combo))]
/// struct Bar(Foo);
/// ```
///
/// With any other arg to diagnostic()
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
///
/// #[derive(Debug, Diagnostic, Error)]
/// enum Variants {
///     #[error("no")]
///     #[diagnostic(transparent, code(invalid::combo))]
///     One(Foo),
/// }
/// ```
///
#[allow(dead_code)]
#[doc(hidden)]
struct TransparentCombinations;

/// Forwarding without overriding the code (struct)
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(forward(0))]
/// struct Bar(Foo);
/// ```
///
/// Forwarding without overriding the code (enum)
///
/// ```compile_fail
/// use thiserror::Error;
/// use miette_derive::Diagnostic;
/// #[derive(Debug, Diagnostic, Error)]
/// #[error("welp")]
/// #[diagnostic(code(foo::bar::baz))]
/// struct Foo {}
/// #[derive(Debug, Diagnostic, Error)]
/// enum Enum {
/// #[error("welp")]
/// #[diagnostic(forward(0))]
/// Bar(Foo) }
/// ```
///
#[allow(dead_code)]
#[doc(hidden)]
struct ForwardWithoutCode;
