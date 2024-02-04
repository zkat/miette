use miette::{Diagnostic, Report, Severity, SourceSpan};
use thiserror::Error;

#[test]
fn related() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo {
        #[related]
        related: Vec<Baz>,
    }

    #[derive(Error, Debug, Diagnostic)]
    enum Bar {
        #[error("variant1")]
        #[diagnostic(code(foo::bar::baz))]
        #[allow(dead_code)]
        Bad {
            #[related]
            related: Vec<Baz>,
        },

        #[error("variant2")]
        #[diagnostic(code(foo::bar::baz))]
        #[allow(dead_code)]
        LessBad(#[related] Vec<Baz>),
    }

    #[derive(Error, Debug, Diagnostic)]
    #[error("welp2")]
    struct Baz;
}

#[test]
fn related_report() {
    #[derive(Error, Debug, Diagnostic)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo {
        #[related]
        related: Vec<Report>,
    }
}

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

    assert_eq!("foo::bar::baz".to_string(), Foo.code().unwrap().to_string());

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

    assert_eq!("foo::x".to_string(), Foo::X.code().unwrap().to_string());
    assert_eq!("foo::y".to_string(), Foo::Y(1).code().unwrap().to_string());
    assert_eq!(
        "foo::z".to_string(),
        Foo::Z { prop: "bar".into() }.code().unwrap().to_string()
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

    assert_eq!(
        "foo::bar::baz".to_string(),
        FooStruct.code().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code("foo::x"))]
        X,
    }

    assert_eq!("foo::x".to_string(), FooEnum::X.code().unwrap().to_string());
}

#[test]
fn path_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct FooStruct;

    assert_eq!(
        "foo::bar::baz".to_string(),
        FooStruct.code().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x))]
        X,
    }

    assert_eq!("foo::x".to_string(), FooEnum::X.code().unwrap().to_string());
}

#[test]
fn path_severity() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), severity("warning"))]
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
    #[diagnostic(code(foo::bar::baz), help("{} x {0} x {:?}", 1, "2"))]
    struct FooStruct<'a>(&'a str);

    assert_eq!(
        "1 x hello x \"2\"".to_string(),
        FooStruct("hello").help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), help("{} x {my_field} x {:?}", 1, "2"))]
    struct BarStruct<'a> {
        my_field: &'a str,
    }

    assert_eq!(
        "1 x hello x \"2\"".to_string(),
        BarStruct { my_field: "hello" }.help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum<'a> {
        #[diagnostic(code(foo::x), help("{} x {0} x {:?}", 1, "2"))]
        X(&'a str),

        #[diagnostic(code(foo::x), help("{} x {len} x {:?}", 1, "2"))]
        Y { len: usize },

        #[diagnostic(code(foo::x), help("{} x {self:?} x {:?}", 1, "2"))]
        Z,
    }

    assert_eq!(
        "1 x bar x \"2\"".to_string(),
        FooEnum::X("bar").help().unwrap().to_string()
    );

    assert_eq!(
        "1 x 10 x \"2\"".to_string(),
        FooEnum::Y { len: 10 }.help().unwrap().to_string()
    );

    assert_eq!(
        "1 x Z x \"2\"".to_string(),
        FooEnum::Z.help().unwrap().to_string()
    );
}

#[test]
fn help_field() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic()]
    struct Foo<'a> {
        #[help]
        do_this: Option<&'a str>,
    }

    assert_eq!(
        "x".to_string(),
        Foo { do_this: Some("x") }.help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic()]
    enum Bar<'a> {
        A(#[help] Option<&'a str>),
        B {
            #[help]
            do_this: Option<&'a str>,
        },
    }

    assert_eq!(
        "x".to_string(),
        Bar::A(Some("x")).help().unwrap().to_string()
    );
    assert_eq!(
        "x".to_string(),
        Bar::B { do_this: Some("x") }.help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic()]
    struct Baz<'a>(#[help] Option<&'a str>);

    assert_eq!("x".to_string(), Baz(Some("x")).help().unwrap().to_string());

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic()]
    struct Quux<'a>(#[help] &'a str);

    assert_eq!("x".to_string(), Quux("x").help().unwrap().to_string());
}

#[test]
fn test_snippet_named_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo<'a> {
        #[source_code]
        src: &'a str,
        #[label("var 1")]
        var1: SourceSpan,
        #[label = "var 2"]
        // Anything that's Clone + Into<SourceSpan> can be used here.
        var2: (usize, usize),
        #[label]
        var3: (usize, usize),
        #[label("var 4")]
        var4: Option<(usize, usize)>,
        #[label]
        var5: Option<(usize, usize)>,
    }
}

#[test]
fn test_snippet_unnamed_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo<'a>(
        #[source_code] &'a str,
        #[label("{0}")] SourceSpan,
        #[label = "idk"] SourceSpan,
        #[label] SourceSpan,
        #[label("foo")] Option<SourceSpan>,
        #[label] Option<SourceSpan>,
    );
}

#[test]
fn test_snippet_enum() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[allow(dead_code)]
    enum Foo<'a> {
        #[diagnostic(code(foo::a))]
        A {
            #[source_code]
            src: &'a str,
            msg: String,
            #[label("hi this is where the thing went wrong ({msg})")]
            var0: SourceSpan,
            #[label = "blorp"]
            var1: SourceSpan,
            #[label]
            var2: SourceSpan,
            #[label("var 3")]
            var3: Option<(usize, usize)>,
            #[label]
            var4: Option<(usize, usize)>,
        },
        #[diagnostic(code(foo::b))]
        B(
            #[source_code] String,
            String,
            #[label("{1}")] SourceSpan,
            #[label = "blorp"] SourceSpan,
            #[label] SourceSpan,
            #[label("foo")] Option<SourceSpan>,
            #[label] Option<SourceSpan>,
        ),
    }
}

#[test]
fn url_basic() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), url("https://example.com/foo/bar"))]
    struct Foo {}

    assert_eq!(
        "https://example.com/foo/bar".to_string(),
        Foo {}.url().unwrap().to_string()
    );
}

#[test]
fn url_docsrs() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), url(docsrs))]
    struct Foo {}

    assert_eq!(
        format!(
            "https://docs.rs/miette/{}/miette/struct.Foo.html",
            env!("CARGO_PKG_VERSION")
        ),
        Foo {}.url().unwrap().to_string()
    );
}

const SNIPPET_TEXT: &str = "hello from miette";

#[derive(Debug, Diagnostic, Error)]
#[error("welp")]
#[diagnostic(
    // code not necessary.
    // code(foo::bar::baz),
    url("https://example.com"),
    help("help"),
    severity(Warning)
)]
struct ForwardsTo {
    #[source_code]
    src: String,
    #[label("highlight text")]
    label: miette::SourceSpan,
}

impl ForwardsTo {
    fn new() -> Self {
        ForwardsTo {
            src: SNIPPET_TEXT.into(),
            label: SourceSpan::new(11.into(), 6),
        }
    }
}

fn check_all(diag: &impl Diagnostic) {
    // check Diagnostic impl forwards all these methods
    assert_eq!(diag.code().as_ref().map(|x| x.to_string()), None);
    assert_eq!(diag.url().unwrap().to_string(), "https://example.com");
    assert_eq!(diag.help().unwrap().to_string(), "help");
    assert_eq!(diag.severity().unwrap(), miette::Severity::Warning);
}

#[test]
fn test_transparent_enum_unnamed() {
    #[derive(Debug, Diagnostic, Error)]
    enum Enum {
        #[error("enum")]
        #[diagnostic(transparent)]
        FooVariant(#[from] ForwardsTo),
    }

    let variant = Enum::FooVariant(ForwardsTo::new());

    check_all(&variant);
}

#[test]
fn test_transparent_enum_named() {
    #[derive(Debug, Diagnostic, Error)]
    enum Enum {
        #[error("enum")]
        #[diagnostic(transparent)]
        FooVariant {
            #[from]
            single_field: ForwardsTo,
        },
        #[error("foo")]
        #[diagnostic(code(foo::bar::bar_variant))]
        BarVariant,
    }

    let variant = Enum::FooVariant {
        single_field: ForwardsTo::new(),
    };

    check_all(&variant);

    let bar_variant = Enum::BarVariant;
    assert_eq!(
        bar_variant.code().unwrap().to_string(),
        "foo::bar::bar_variant"
    );
}

#[test]
fn test_transparent_struct_named() {
    #[derive(Debug, Diagnostic, Error)]
    #[error(transparent)]
    #[diagnostic(transparent)]
    struct Struct {
        #[from]
        single_field: ForwardsTo,
    }
    // Also check the From impl here
    let variant: Struct = ForwardsTo::new().into();
    check_all(&variant);
}

#[test]
fn test_transparent_struct_unnamed() {
    #[derive(Debug, Diagnostic, Error)]
    #[error(transparent)]
    #[diagnostic(transparent)]
    struct Struct(#[from] ForwardsTo);
    let variant = Struct(ForwardsTo::new());
    check_all(&variant);
}

#[test]
fn test_forward_struct_named() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("display")]
    #[diagnostic(
        code(foo::bar::overridden),
        severity(Advice),
        help("{help}"),
        forward(span)
    )]
    struct Struct<'a> {
        span: ForwardsTo,
        help: &'a str,
    }
    // Also check the From impl here
    let diag = Struct {
        span: ForwardsTo::new(),
        help: "overridden help please",
    };
    assert_eq!(diag.code().unwrap().to_string(), "foo::bar::overridden");
    assert_eq!(diag.help().unwrap().to_string(), "overridden help please");
    assert_eq!(diag.severity(), Some(Severity::Advice));
}

#[test]
fn test_forward_struct_unnamed() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("display")]
    #[diagnostic(code(foo::bar::overridden), url("{1}"), forward(0))]
    struct Struct<'a>(ForwardsTo, &'a str);

    // Also check the From impl here
    let diag = Struct(ForwardsTo::new(), "url here");
    assert_eq!(diag.code().unwrap().to_string(), "foo::bar::overridden");
    assert_eq!(diag.url().unwrap().to_string(), "url here");
}

#[test]
fn test_forward_enum_named() {
    #[derive(Debug, Diagnostic, Error)]
    enum Enum<'a> {
        #[error("help: {help_text}")]
        #[diagnostic(code(foo::bar::overridden), help("{help_text}"), forward(span))]
        Variant {
            span: ForwardsTo,
            help_text: &'a str,
        },
    }
    // Also check the From impl here
    let variant: Enum = Enum::Variant {
        span: ForwardsTo::new(),
        help_text: "overridden help please",
    };
    assert_eq!(variant.code().unwrap().to_string(), "foo::bar::overridden");
    assert_eq!(
        variant.help().unwrap().to_string(),
        "overridden help please"
    );
}

#[test]
fn test_forward_enum_unnamed() {
    #[derive(Debug, Diagnostic, Error)]
    enum ForwardEnumUnnamed<'a> {
        #[error("help: {1}")]
        #[diagnostic(code(foo::bar::overridden), help("{1}"), forward(0))]
        Variant(ForwardsTo, &'a str),
    }
    // Also check the From impl here
    let variant = ForwardEnumUnnamed::Variant(ForwardsTo::new(), "overridden help please");
    assert_eq!(variant.code().unwrap().to_string(), "foo::bar::overridden");
    assert_eq!(
        variant.help().unwrap().to_string(),
        "overridden help please"
    );
}

#[test]
fn test_unit_struct_display() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("unit only")]
    #[diagnostic(code(foo::bar::overridden), help("hello from unit help"))]
    struct UnitOnly;
    assert_eq!(UnitOnly.help().unwrap().to_string(), "hello from unit help");
}

#[test]
fn test_unit_enum_display() {
    #[derive(Debug, Diagnostic, Error)]
    enum Enum {
        #[error("unit only")]
        #[diagnostic(code(foo::bar::overridden), help("hello from unit help"))]
        UnitVariant,
    }
    assert_eq!(
        Enum::UnitVariant.help().unwrap().to_string(),
        "hello from unit help"
    );
}

#[test]
fn test_optional_source_code() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("struct with optional source")]
    struct Struct {
        #[source_code]
        src: Option<String>,
    }
    assert!(Struct { src: None }.source_code().is_none());
    assert!(Struct {
        src: Some("".to_string())
    }
    .source_code()
    .is_some());

    #[derive(Debug, Diagnostic, Error)]
    enum Enum {
        #[error("variant1 with optional source")]
        Variant1 {
            #[source_code]
            src: Option<String>,
        },
        #[error("variant2 with optional source")]
        Variant2 {
            #[source_code]
            src: Option<String>,
        },
    }
    assert!(Enum::Variant1 { src: None }.source_code().is_none());
    assert!(Enum::Variant1 {
        src: Some("".to_string())
    }
    .source_code()
    .is_some());
    assert!(Enum::Variant2 { src: None }.source_code().is_none());
    assert!(Enum::Variant2 {
        src: Some("".to_string())
    }
    .source_code()
    .is_some());
}
