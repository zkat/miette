/// Return early with an error.
///
/// This macro is equivalent to `return Err(From::from($err))`.
///
/// # Example
///
/// ```
/// # use miette::{bail, Result};
/// #
/// # fn has_permission(user: usize, resource: usize) -> bool {
/// #     true
/// # }
/// #
/// # fn main() -> Result<()> {
/// #     let user = 0;
/// #     let resource = 0;
/// #
/// if !has_permission(user, resource) {
///     bail!("permission denied for accessing {}", resource);
/// }
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// # use miette::{bail, Result};
/// # use thiserror::Error;
/// #
/// # const MAX_DEPTH: usize = 1;
/// #
/// #[derive(Error, Debug)]
/// enum ScienceError {
///     #[error("recursion limit exceeded")]
///     RecursionLimitExceeded,
///     # #[error("...")]
///     # More = (stringify! {
///     ...
///     # }, 1).1,
/// }
///
/// # fn main() -> Result<()> {
/// #     let depth = 0;
/// #     let err: &'static dyn std::error::Error = &ScienceError::RecursionLimitExceeded;
/// #
/// if depth > MAX_DEPTH {
///     bail!(ScienceError::RecursionLimitExceeded);
/// }
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// use miette::{bail, Result, Severity};
///
/// fn divide(x: f64, y: f64) -> Result<f64> {
///     if y.abs() < 1e-3 {
///         bail!("dividing by value close to 0", severity = Severity::Warning);
///     }
///     Ok(x / y)
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::private::Err($crate::miette!($msg));
    };
    ($err:expr $(,)?) => {
        return $crate::private::Err($crate::miette!($err));
    };
    ($fmt:expr $(, $key:ident = $value:expr)* $(,)?) => {
        return $crate::private::Err($crate::miette!($fmt, $($key = $value),*));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::private::Err($crate::miette!($fmt, $($arg)*));
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return Err(From::from($err)); }`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
///
/// # Example
///
/// ```
/// # use miette::{ensure, Result};
/// #
/// # fn main() -> Result<()> {
/// #     let user = 0;
/// #
/// ensure!(user == 0, "only user 0 is allowed");
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// # use miette::{ensure, Result};
/// # use thiserror::Error;
/// #
/// # const MAX_DEPTH: usize = 1;
/// #
/// #[derive(Error, Debug)]
/// enum ScienceError {
///     #[error("recursion limit exceeded")]
///     RecursionLimitExceeded,
///     # #[error("...")]
///     # More = (stringify! {
///     ...
///     # }, 1).1,
/// }
///
/// # fn main() -> Result<()> {
/// #     let depth = 0;
/// #
/// ensure!(depth <= MAX_DEPTH, ScienceError::RecursionLimitExceeded);
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// use miette::{ensure, Result, Severity};
///
/// fn divide(x: f64, y: f64) -> Result<f64> {
///     ensure!(
///         y.abs() >= 1e-3,
///         "dividing by value close to 0",
///         severity = Severity::Warning
///     );
///     Ok(x / y)
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::miette!($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::miette!($err));
        }
    };
    ($cond:expr, $fmt:expr $(, $key:ident = $value:expr)* $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::miette!($fmt, $($key = $value),*));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::miette!($fmt, $($arg)*));
        }
    };
}

/// Construct an ad-hoc [`Report`].
///
/// # Examples
///
/// With string literal and interpolation:
/// ```
/// # type V = ();
/// #
/// use miette::{miette, Result};
///
/// fn lookup(key: &str) -> Result<V> {
///     if key.len() != 16 {
///         return Err(miette!("key length must be 16 characters, got {:?}", key));
///     }
///
///     // ...
///     # Ok(())
/// }
/// ```
///
/// With [`MietteDiagnostic`]-like arguments:
/// ```
/// use miette::{miette, LabeledSpan, Severity};
///
/// let source = "(2 + 2".to_string();
/// let report = miette!(
///     "expected closing ')'",
///     // Those fields are optional
///     severity = Severity::Error,
///     code = "expected::rparen",
///     help = "always close your parens",
///     labels = vec![LabeledSpan::at_offset(6, "here")],
///     url = "https://example.com"
/// )
/// .with_source_code(source);
/// ```
///
/// ## `anyhow`/`eyre` Users
///
/// You can just replace `use`s of the `anyhow!`/`eyre!` macros with `miette!`.
#[macro_export]
macro_rules! miette {
    ($msg:literal $(,)?) => {
        // Handle $:literal as a special case to make cargo-expanded code more
        // concise in the common case.
        $crate::private::new_adhoc($msg)
    };
    ($err:expr $(,)?) => ({
        use $crate::private::kind::*;
        let error = $err;
        (&error).miette_kind().new(error)
    });
    ($fmt:expr $(, $key:ident = $value:expr)* $(,)?) => {
        $crate::Report::from($crate::diagnostic!($fmt, $($key = $value,)*))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::private::new_adhoc(format!($fmt, $($arg)*))
    };
}

/// Construct a [`MietteDiagnostic`] in more user-friendly way.
///
/// # Examples
/// ```
/// use miette::{diagnostic, LabeledSpan, Severity};
///
/// let source = "(2 + 2".to_string();
/// let diag = diagnostic!(
///     "expected closing ')'",
///     // Those fields are optional
///     severity = Severity::Error,
///     code = "expected::rparen",
///     help = "always close your parens",
///     labels = vec![LabeledSpan::at_offset(6, "here")],
///     url = "https://example.com"
/// );
/// ```
#[macro_export]
macro_rules! diagnostic {
    ($fmt:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let mut diag = $crate::MietteDiagnostic::new(format!("{}", $fmt));
        $(diag.$key = Some($value.into());)*
        diag
    }};
}
