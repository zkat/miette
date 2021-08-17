use std::sync::Arc;

use miette::{Diagnostic, Severity, SourceSpan};
use thiserror::Error;

#[test]
fn basic_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(
        code = "foo::bar::baz",
        severity = "Error",
        help = "try doing it better"
    )]
    struct Foo;

    assert_eq!("foo::bar::baz".to_string(), Foo.code().to_string());

    assert_eq!(Some(Severity::Error), Foo.severity());

    assert_eq!(
        "try doing it better".to_string(),
        Foo.help().unwrap().to_string()
    );
}

#[test]
fn basic_enum() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum Foo {
        #[diagnostic(
            code = "foo::x",
            severity = "Warning",
            help = "Try using Foo::Y instead"
        )]
        X,
        #[diagnostic(code = "foo::y")]
        Y(usize),
        #[diagnostic(code = "foo::z")]
        Z { prop: String },
    }

    assert_eq!("foo::x".to_string(), Foo::X.code().to_string());
    assert_eq!("foo::y".to_string(), Foo::Y(1).code().to_string());
    assert_eq!(
        "foo::z".to_string(),
        Foo::Z { prop: "bar".into() }.code().to_string()
    );

    assert_eq!(Some(Severity::Warning), Foo::X.severity());
    assert_eq!(None, Foo::Y(1).severity());
}

#[test]
fn paren_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code("foo::bar::baz"))]
    struct FooStruct;

    assert_eq!("foo::bar::baz".to_string(), FooStruct.code().to_string());

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code("foo::x"))]
        X,
    }

    assert_eq!("foo::x".to_string(), FooEnum::X.code().to_string());
}

#[test]
fn path_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct FooStruct;

    assert_eq!("foo::bar::baz".to_string(), FooStruct.code().to_string());

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x))]
        X,
    }

    assert_eq!("foo::x".to_string(), FooEnum::X.code().to_string());
}

#[test]
fn path_severity() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), severity(Warning))]
    struct FooStruct;

    assert_eq!(Some(Severity::Warning), FooStruct.severity());

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x), severity(Warning))]
        X,
    }

    assert_eq!(Some(Severity::Warning), FooEnum::X.severity());
}

#[test]
fn list_help() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), help("try doing it better"))]
    struct FooStruct;

    assert_eq!(
        "try doing it better".to_string(),
        FooStruct.help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x), help("try doing it better"))]
        X,
    }

    assert_eq!(
        "try doing it better".to_string(),
        FooEnum::X.help().unwrap().to_string()
    );
}

#[test]
fn fmt_help() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(
        code(foo::bar::baz),
        help("{} {}", 1, self.0),
    )]
    struct FooStruct(String);

    assert_eq!(
        "1 hello".to_string(),
        FooStruct("hello".into()).help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x), help("{} {}", 1, "bar"))]
        X,
    }

    assert_eq!("1 bar".to_string(), FooEnum::X.help().unwrap().to_string());
}

#[test]
fn test_snippet_named_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo {
        // The actual "source code" our contexts will be using. This can be
        // reused by multiple contexts!
        //
        // The `Arc` is so you don't have to clone the entire thing into this
        // Diagnostic. We just need to be able to read it~
        src: Arc<String>,

        // The "snippet" span. This is the span that will be displayed to
        // users. It should be a big enough slice of the Source to provide
        // reasonable context, but still somewhat compact.
        //
        // You can have as many of these #[snippet] fields as you want, and
        // even feed them from different sources!
        //
        // Example display:
        //   / [my_snippet]: hi this is where the thing went wrong.
        // 1 | hello
        // 2 |     world
        #[snippet(src, "hi this is where the thing went wrong")]
        snip: SourceSpan, // Defines filename using `label`

        // "Highlights" are the specific highlights _inside_ the snippet.
        // These will be used to underline/point to specific sections of the
        // #[snippet] they refer to. As such, these SourceSpans must be within
        // the bounds of their referenced snippet.
        //
        // Example display:
        // 1 | var1 + var2
        //   | ^^^^   ^^^^ - var 2
        //   | |
        //   | var 1
        #[highlight(snip)]
        var1: SourceSpan, // label from SourceSpan is used, if any.
        #[highlight(snip)]
        var2: SourceSpan,

        // Now with member source names
        filename: String,
        second_message: String,
        #[snippet(src, filename, second_message)]
        snip2: SourceSpan,
    }
}

#[test]
fn test_snippet_unnamed_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo(
        Arc<String>,
        #[snippet(0, "hi")] SourceSpan,
        #[highlight(1)] SourceSpan,
        #[highlight(1)] SourceSpan,
        // referenced source name
        String,
        #[snippet(0, 4)] SourceSpan,
        #[highlight(5)] SourceSpan,
        #[highlight(5)] SourceSpan,
    );
}

#[test]
fn test_snippet_enum() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[allow(dead_code)]
    enum Foo {
        #[diagnostic(code(foo::a))]
        A {
            src: Arc<String>,
            #[snippet(src, "my_snippet.rs", "hi this is where the thing went wrong")]
            snip: SourceSpan,
            #[highlight(snip, "var 1")]
            var1: SourceSpan,
            #[highlight(snip, "var 2")]
            var2: SourceSpan,
            filename: String,
            second_message: String,
            #[snippet(src, filename, second_message)]
            snip2: SourceSpan,
        },
        #[diagnostic(code(foo::b))]
        B(
            Arc<String>,
            #[snippet(0, "my_snippet.rs", "hi")] SourceSpan,
            #[highlight(1, "var 1")] SourceSpan,
            #[highlight(1, "var 2")] SourceSpan,
            // referenced source name
            String,
            String,
            #[snippet(0, 4, 5)] SourceSpan,
            #[highlight(6, "var 3")] SourceSpan,
            #[highlight(6, "var 4")] SourceSpan,
        ),
    }
}
