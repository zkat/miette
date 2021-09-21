use thiserror::Error;

use crate::{self as miette, Context, Diagnostic, Result};

/// Tells miette to render panics using its rendering engine.
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let mut message = "Something went wrong".to_string();
        let payload = info.payload();
        if let Some(msg) = payload.downcast_ref::<&str>() {
            message = msg.to_string();
        }
        if let Some(msg) = payload.downcast_ref::<String>() {
            message = msg.clone();
        }
        let mut report: Result<()> = Err(Panic(message).into());
        if let Some(loc) = info.location() {
            report = report
                .with_context(|| format!("at {}:{}:{}", loc.file(), loc.line(), loc.column()));
        }
        if let Err(err) = report.with_context(|| "Main thread panicked.".to_string()) {
            eprintln!("Error: {:?}", err);
        }
    }));
}

#[derive(Debug, Error, Diagnostic)]
#[error("{0}")]
#[diagnostic(help("set the `RUST_BACKTRACE=1` environment variable to display a backtrace."))]
struct Panic(String);
