#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![allow(
    clippy::needless_doctest_main,
    clippy::new_ret_no_self,
    clippy::wrong_self_convention
)]
use core::fmt::Display;
use core::mem::ManuallyDrop;

use std::error::Error as StdError;

use atty::Stream;
use once_cell::sync::OnceCell;

#[doc(hidden)]
pub use ReportHandler as EyreContext;
#[doc(hidden)]
pub use Report as ErrReport;
/// Compatibility re-export of `Report` for interop with `anyhow`
pub use Report as Error;
/// Compatibility re-export of `WrapErr` for interop with `anyhow`
pub use WrapErr as Context;
pub use into_diagnostic::*;

use error::ErrorImpl;
use crate::{Diagnostic, GraphicalReportHandler, NarratableReportHandler};

mod context;
mod error;
mod fmt;
mod into_diagnostic;
mod kind;
mod macros;
mod wrapper;

/**
Core Diagnostic wrapper type.
*/
pub struct Report {
    inner: ManuallyDrop<Box<ErrorImpl<()>>>,
}

type ErrorHook =
    Box<dyn Fn(&(dyn Diagnostic + 'static)) -> Box<dyn ReportHandler> + Sync + Send + 'static>;

static HOOK: OnceCell<ErrorHook> = OnceCell::new();

/// Error indicating that `set_hook` was unable to install the provided ErrorHook
#[derive(Debug)]
pub struct InstallError;

impl core::fmt::Display for InstallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("cannot install provided ErrorHook, a hook has already been installed")
    }
}

impl StdError for InstallError {}
impl Diagnostic for InstallError {}

/**
Set the hook?
*/
pub fn set_hook(hook: ErrorHook) -> Result<(), InstallError> {
    HOOK.set(hook).map_err(|_| InstallError)
}

#[cfg_attr(track_caller, track_caller)]
#[cfg_attr(not(track_caller), allow(unused_mut))]
fn capture_handler(error: &(dyn Diagnostic + 'static)) -> Box<dyn ReportHandler> {
    let hook = HOOK.get_or_init(|| Box::new(get_default_printer)).as_ref();

    let mut handler = hook(error);

    #[cfg(track_caller)]
    {
        handler.track_caller(std::panic::Location::caller())
    }

    #[allow(clippy::let_and_return)]
    handler
}

fn get_default_printer(_err: &(dyn Diagnostic + 'static)) -> Box<dyn ReportHandler + 'static> {
    let fancy = if let Ok(string) = std::env::var("NO_COLOR") {
        string == "0"
    } else if let Ok(string) = std::env::var("CLICOLOR") {
        string != "0" || string == "1"
    } else {
        atty::is(Stream::Stdout) && atty::is(Stream::Stderr) && !ci_info::is_ci()
    };
    if fancy {
        Box::new(GraphicalReportHandler::new())
    } else {
        Box::new(NarratableReportHandler)
    }
}

impl dyn ReportHandler {
    ///
    pub fn is<T: ReportHandler>(&self) -> bool {
        // Get `TypeId` of the type this function is instantiated with.
        let t = core::any::TypeId::of::<T>();

        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();

        // Compare both `TypeId`s on equality.
        t == concrete
    }

    ///
    pub fn downcast_ref<T: ReportHandler>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn ReportHandler as *const T)) }
        } else {
            None
        }
    }

    ///
    pub fn downcast_mut<T: ReportHandler>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn ReportHandler as *mut T)) }
        } else {
            None
        }
    }
}

/// Error Report Handler trait for customizing `eyre::Report`
pub trait ReportHandler: core::any::Any + Send + Sync {
    /// Define the report format
    ///
    /// Used to override the report format of `eyre::Report`
    ///
    /// # Example
    ///
    /// ```rust
    /// use backtrace::Backtrace;
    /// use eyre::EyreHandler;
    /// use eyre::Chain;
    /// use std::error::Error;
    /// use indenter::indented;
    ///
    /// pub struct Handler {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl EyreHandler for Handler {
    ///     fn debug(
    ///         &self,
    ///         error: &(dyn Error + 'static),
    ///         f: &mut core::fmt::Formatter<'_>,
    ///     ) -> core::fmt::Result {
    ///         use core::fmt::Write as _;
    ///
    ///         if f.alternate() {
    ///             return core::fmt::Debug::fmt(error, f);
    ///         }
    ///
    ///         write!(f, "{}", error)?;
    ///
    ///         if let Some(cause) = error.source() {
    ///             write!(f, "\n\nCaused by:")?;
    ///             let multiple = cause.source().is_some();
    ///
    ///             for (n, error) in Chain::new(cause).enumerate() {
    ///                 writeln!(f)?;
    ///                 if multiple {
    ///                     write!(indented(f).ind(n), "{}", error)?;
    ///                 } else {
    ///                     write!(indented(f), "{}", error)?;
    ///                 }
    ///             }
    ///         }
    ///
    ///         let backtrace = &self.backtrace;
    ///         write!(f, "\n\nStack backtrace:\n{:?}", backtrace)?;
    ///
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn debug(
        &self,
        error: &(dyn Diagnostic + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result;

    /// Override for the `Display` format
    fn display(
        &self,
        error: &(dyn StdError + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", error)?;

        if f.alternate() {
            for cause in crate::chain::Chain::new(error).skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }

    /// Store the location of the caller who constructed this error report
    #[allow(unused_variables)]
    fn track_caller(&mut self, location: &'static std::panic::Location<'static>) {}
}

/// Iterator of a chain of source errors.
///
/// This type is the iterator returned by [`Report::chain`].
///
/// # Example
///
/// ```
/// use eyre::Report;
/// use std::io;
///
/// pub fn underlying_io_error_kind(error: &Report) -> Option<io::ErrorKind> {
///     for cause in error.chain() {
///         if let Some(io_error) = cause.downcast_ref::<io::Error>() {
///             return Some(io_error.kind());
///         }
///     }
///     None
/// }
/// ```
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Chain<'a> {
    state: crate::chain::ChainState<'a>,
}

/// type alias for `Result<T, Report>`
///
/// This is a reasonable return type to use throughout your application but also for `fn main`; if
/// you do, failures will be printed along with a backtrace if one was captured.
///
/// `eyre::Result` may be used with one *or* two type parameters.
///
/// ```rust
/// use eyre::Result;
///
/// # const IGNORE: &str = stringify! {
/// fn demo1() -> Result<T> {...}
///            // ^ equivalent to std::result::Result<T, eyre::Error>
///
/// fn demo2() -> Result<T, OtherError> {...}
///            // ^ equivalent to std::result::Result<T, OtherError>
/// # };
/// ```
///
/// # Example
///
/// ```
/// # pub trait Deserialize {}
/// #
/// # mod serde_json {
/// #     use super::Deserialize;
/// #     use std::io;
/// #
/// #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct ClusterMap;
/// #
/// # impl Deserialize for ClusterMap {}
/// #
/// use eyre::Result;
///
/// fn main() -> Result<()> {
///     # return Ok(());
///     let config = std::fs::read_to_string("cluster.json")?;
///     let map: ClusterMap = serde_json::from_str(&config)?;
///     println!("cluster info: {:#?}", map);
///     Ok(())
/// }
/// ```
pub type Result<T, E = Report> = core::result::Result<T, E>;

/// Provides the `wrap_err` method for `Result`.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `eyre`.
///
/// # Example
///
/// ```
/// use eyre::{WrapErr, Result};
/// use std::fs;
/// use std::path::PathBuf;
///
/// pub struct ImportantThing {
///     path: PathBuf,
/// }
///
/// impl ImportantThing {
///     # const IGNORE: &'static str = stringify! {
///     pub fn detach(&mut self) -> Result<()> {...}
///     # };
///     # fn detach(&mut self) -> Result<()> {
///     #     unimplemented!()
///     # }
/// }
///
/// pub fn do_it(mut it: ImportantThing) -> Result<Vec<u8>> {
///     it.detach().wrap_err("Failed to detach the important thing")?;
///
///     let path = &it.path;
///     let content = fs::read(path)
///         .wrap_err_with(|| format!("Failed to read instrs from {}", path.display()))?;
///
///     Ok(content)
/// }
/// ```
///
/// When printed, the outermost error would be printed first and the lower
/// level underlying causes would be enumerated below.
///
/// ```console
/// Error: Failed to read instrs from ./path/to/instrs.json
///
/// Caused by:
///     No such file or directory (os error 2)
/// ```
///
/// # Wrapping Types That Don't impl `Error` (e.g. `&str` and `Box<dyn Error>`)
///
/// Due to restrictions for coherence `Report` cannot impl `From` for types that don't impl
/// `Error`. Attempts to do so will give "this type might implement Error in the future" as an
/// error. As such, `wrap_err`, which uses `From` under the hood, cannot be used to wrap these
/// types. Instead we encourage you to use the combinators provided for `Result` in `std`/`core`.
///
/// For example, instead of this:
///
/// ```rust,compile_fail
/// use std::error::Error;
/// use eyre::{WrapErr, Report};
///
/// fn wrap_example(err: Result<(), Box<dyn Error + Send + Sync + 'static>>) -> Result<(), Report> {
///     err.wrap_err("saw a downstream error")
/// }
/// ```
///
/// We encourage you to write this:
///
/// ```rust
/// use std::error::Error;
/// use eyre::{WrapErr, Report, eyre};
///
/// fn wrap_example(err: Result<(), Box<dyn Error + Send + Sync + 'static>>) -> Result<(), Report> {
///     err.map_err(|e| eyre!(e)).wrap_err("saw a downstream error")
/// }
/// ```
///
/// # Effect on downcasting
///
/// After attaching a message of type `D` onto an error of type `E`, the resulting
/// `eyre::Error` may be downcast to `D` **or** to `E`.
///
/// That is, in codebases that rely on downcasting, Eyre's wrap_err supports
/// both of the following use cases:
///
///   - **Attaching messages whose type is insignificant onto errors whose type
///     is used in downcasts.**
///
///     In other error libraries whose wrap_err is not designed this way, it can
///     be risky to introduce messages to existing code because new message might
///     break existing working downcasts. In Eyre, any downcast that worked
///     before adding the message will continue to work after you add a message, so
///     you should freely wrap errors wherever it would be helpful.
///
///     ```
///     # use eyre::bail;
///     # use thiserror::Error;
///     #
///     # #[derive(Error, Debug)]
///     # #[error("???")]
///     # struct SuspiciousError;
///     #
///     # fn helper() -> Result<()> {
///     #     bail!(SuspiciousError);
///     # }
///     #
///     use eyre::{WrapErr, Result};
///
///     fn do_it() -> Result<()> {
///         helper().wrap_err("Failed to complete the work")?;
///         # const IGNORE: &str = stringify! {
///         ...
///         # };
///         # unreachable!()
///     }
///
///     fn main() {
///         let err = do_it().unwrap_err();
///         if let Some(e) = err.downcast_ref::<SuspiciousError>() {
///             // If helper() returned SuspiciousError, this downcast will
///             // correctly succeed even with the message in between.
///             # return;
///         }
///         # panic!("expected downcast to succeed");
///     }
///     ```
///
///   - **Attaching message whose type is used in downcasts onto errors whose
///     type is insignificant.**
///
///     Some codebases prefer to use machine-readable messages to categorize
///     lower level errors in a way that will be actionable to higher levels of
///     the application.
///
///     ```
///     # use eyre::bail;
///     # use thiserror::Error;
///     #
///     # #[derive(Error, Debug)]
///     # #[error("???")]
///     # struct HelperFailed;
///     #
///     # fn helper() -> Result<()> {
///     #     bail!("no such file or directory");
///     # }
///     #
///     use eyre::{WrapErr, Result};
///
///     fn do_it() -> Result<()> {
///         helper().wrap_err(HelperFailed)?;
///         # const IGNORE: &str = stringify! {
///         ...
///         # };
///         # unreachable!()
///     }
///
///     fn main() {
///         let err = do_it().unwrap_err();
///         if let Some(e) = err.downcast_ref::<HelperFailed>() {
///             // If helper failed, this downcast will succeed because
///             // HelperFailed is the message that has been attached to
///             // that error.
///             # return;
///         }
///         # panic!("expected downcast to succeed");
///     }
///     ```
pub trait WrapErr<T, E>: context::private::Sealed {
    /// Wrap the error value with a new adhoc error
    #[cfg_attr(track_caller, track_caller)]
    fn wrap_err<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Wrap the error value with a new adhoc error that is evaluated lazily
    /// only once an error does occur.
    #[cfg_attr(track_caller, track_caller)]
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Compatibility re-export of wrap_err for interop with `anyhow`
    #[cfg_attr(track_caller, track_caller)]
    fn context<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility re-export of wrap_err_with for interop with `anyhow`
    #[cfg_attr(track_caller, track_caller)]
    fn with_context<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

/// Provides the `context` method for `Option` when porting from `anyhow`
///
/// This trait is sealed and cannot be implemented for types outside of
/// `eyre`.
///
/// ## Why Doesn't `Eyre` impl `WrapErr` for `Option`?
///
/// `eyre` doesn't impl `WrapErr` for `Option` because `wrap_err` implies that you're creating a
/// new error that saves the previous error as its `source`. Calling `wrap_err` on an `Option` is
/// meaningless because there is no source error. `anyhow` avoids this issue by using a different
/// mental model where you're adding "context" to an error, though this not a mental model for
/// error handling that `eyre` agrees with.
///
/// Instead, `eyre` encourages users to think of each error as distinct, where the previous error
/// is the context being saved by the new error, which is backwards compared to anyhow's model. In
/// this model you're encouraged to use combinators provided by `std` for `Option` to convert an
/// option to a `Result`
///
/// # Example
///
/// Instead of:
///
/// ```rust
/// use eyre::ContextCompat;
///
/// fn get_thing(mut things: impl Iterator<Item = u32>) -> eyre::Result<u32> {
///     things
///         .find(|&thing| thing == 42)
///         .context("the thing wasn't in the list")
/// }
/// ```
///
/// We encourage you to use this:
///
/// ```rust
/// use eyre::eyre;
///
/// fn get_thing(mut things: impl Iterator<Item = u32>) -> eyre::Result<u32> {
///     things
///         .find(|&thing| thing == 42)
///         .ok_or_else(|| eyre!("the thing wasn't in the list"))
/// }
/// ```
pub trait ContextCompat<T>: context::private::Sealed {
    /// Compatibility version of `wrap_err` for creating new errors with new source on `Option`
    /// when porting from `anyhow`
    #[cfg_attr(track_caller, track_caller)]
    fn context<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility version of `wrap_err_with` for creating new errors with new source on `Option`
    /// when porting from `anyhow`
    #[cfg_attr(track_caller, track_caller)]
    fn with_context<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Compatibility re-export of `context` for porting from `anyhow` to `eyre`
    #[cfg_attr(track_caller, track_caller)]
    fn wrap_err<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility re-export of `with_context` for porting from `anyhow` to `eyre`
    #[cfg_attr(track_caller, track_caller)]
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

// Not public API. Referenced by macro-generated code.
#[doc(hidden)]
pub mod private {
    use super::Report;
    use core::fmt::{Debug, Display};

    pub use core::result::Result::Err;

    #[doc(hidden)]
    pub mod kind {
        pub use super::super::kind::{AdhocKind, TraitKind};

        pub use super::super::kind::BoxedKind;
    }

    #[cfg_attr(track_caller, track_caller)]
    pub fn new_adhoc<M>(message: M) -> Report
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }
}
