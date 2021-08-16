use miette::{Diagnostic, Severity};
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
        #[diagnostic(
            code(foo::x),
            help("{} {}", 1, "bar"),
        )]
        X,
    }

    assert_eq!(
        "1 bar".to_string(),
        FooEnum::X.help().unwrap().to_string()
    );
}
