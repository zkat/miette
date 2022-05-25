#![cfg(feature = "fancy-no-backtrace")]

use lazy_static::lazy_static;
use miette::{Diagnostic, MietteHandler, MietteHandlerOpts, ReportHandler, RgbColors};
use regex::Regex;
use std::fmt::{self, Debug};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Eq, PartialEq, Debug)]
enum ColorFormat {
    NoColor,
    Ansi,
    Rgb,
}

#[derive(Debug, Diagnostic, Error)]
#[error("oops!")]
#[diagnostic(code(oops::my::bad), help("try doing it better next time?"))]
struct MyBad;

struct FormatTester(MietteHandler);

impl Debug for FormatTester {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.debug(&MyBad, f)
    }
}

/// Check the color format used by a handler.
fn color_format(handler: MietteHandler) -> ColorFormat {
    let out = format!("{:?}", FormatTester(handler));

    let rgb_colors = Regex::new(r"\u{1b}\[[34]8;2;").unwrap();
    let ansi_colors = Regex::new(r"\u{1b}\[(3|4|9|10)[0-7][m;]").unwrap();
    if rgb_colors.is_match(&out) {
        ColorFormat::Rgb
    } else if ansi_colors.is_match(&out) {
        ColorFormat::Ansi
    } else {
        ColorFormat::NoColor
    }
}

/// Runs a function with an environment variable set to a specific value, then
/// sets it back to it's original value once completed.
fn with_env_var<F: FnOnce()>(var: &str, value: &str, body: F) {
    let old_value = std::env::var_os(var);
    std::env::set_var(var, value);
    body();
    if let Some(old_value) = old_value {
        std::env::set_var(var, old_value);
    } else {
        std::env::remove_var(var);
    }
}

lazy_static! {
    static ref COLOR_ENV_VARS: Mutex<()> = Mutex::new(());
}

/// Assert the color format used by a handler with different levels of terminal
/// support.
fn check_colors<F: Fn(MietteHandlerOpts) -> MietteHandlerOpts>(
    make_handler: F,
    no_support: ColorFormat,
    ansi_support: ColorFormat,
    rgb_support: ColorFormat,
) {
    // To simulate different levels of terminal support we're using specific
    // environment variables that are handled by the supports_color crate.
    //
    // Since environment variables are shared for the entire process, we need
    // to ensure that only one test that modifies these env vars runs at a time.
    let guard = COLOR_ENV_VARS.lock().unwrap();

    with_env_var("NO_COLOR", "1", || {
        let handler = make_handler(MietteHandlerOpts::new()).build();
        assert_eq!(color_format(handler), no_support);
    });
    with_env_var("FORCE_COLOR", "1", || {
        let handler = make_handler(MietteHandlerOpts::new()).build();
        assert_eq!(color_format(handler), ansi_support);
    });
    with_env_var("FORCE_COLOR", "3", || {
        let handler = make_handler(MietteHandlerOpts::new()).build();
        assert_eq!(color_format(handler), rgb_support);
    });

    drop(guard);
}

#[test]
fn no_color_preference() {
    use ColorFormat::*;
    check_colors(|opts| opts, NoColor, Ansi, Ansi);
}

#[test]
fn color_never() {
    use ColorFormat::*;
    check_colors(|opts| opts.color(false), NoColor, NoColor, NoColor);
}

#[test]
fn color_always() {
    use ColorFormat::*;
    check_colors(|opts| opts.color(true), Ansi, Ansi, Ansi);
}

#[test]
fn rgb_preferred() {
    use ColorFormat::*;
    check_colors(
        |opts| opts.rgb_colors(RgbColors::Preferred),
        NoColor,
        Ansi,
        Rgb,
    );
}

#[test]
fn rgb_always() {
    use ColorFormat::*;
    check_colors(|opts| opts.rgb_colors(RgbColors::Always), NoColor, Rgb, Rgb);
}

#[test]
fn color_always_rgb_always() {
    use ColorFormat::*;
    check_colors(
        |opts| opts.color(true).rgb_colors(RgbColors::Always),
        Rgb,
        Rgb,
        Rgb,
    );
}
