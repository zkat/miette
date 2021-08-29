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
    struct FooStruct(String);

    assert_eq!(
        "1 x hello x \"2\"".to_string(),
        FooStruct("hello".into()).help().unwrap().to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz), help("{} x {my_field} x {:?}", 1, "2"))]
    struct BarStruct {
        my_field: String,
    }

    assert_eq!(
        "1 x hello x \"2\"".to_string(),
        BarStruct {
            my_field: "hello".into()
        }
        .help()
        .unwrap()
        .to_string()
    );

    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    enum FooEnum {
        #[diagnostic(code(foo::x), help("{} x {0} x {:?}", 1, "2"))]
        X(String),

        #[diagnostic(code(foo::x), help("{} x {len} x {:?}", 1, "2"))]
        Y { len: usize },

        #[diagnostic(code(foo::x), help("{} x {self:?} x {:?}", 1, "2"))]
        Z,
    }

    assert_eq!(
        "1 x bar x \"2\"".to_string(),
        FooEnum::X("bar".into()).help().unwrap().to_string()
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
fn test_snippet_named_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo {
        // The actual "source code" our contexts will be using. This can be
        // reused by multiple contexts, and just needs to implement
        // miette::Source!
        src: String,

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
        #[snippet(src, message("hi this is where the thing went wrong"))]
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
        // label from SourceSpan is used, if any.
        var1: SourceSpan,
        #[highlight(snip)]
        // Anything that's Clone + Into<SourceSpan> can be used here.
        var2: (usize, usize),
    }
}

#[test]
fn test_snippet_unnamed_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("welp")]
    #[diagnostic(code(foo::bar::baz))]
    struct Foo(
        String,
        #[snippet(0, message("hi"))] SourceSpan,
        #[highlight(1)] SourceSpan,
        #[highlight(1)] SourceSpan,
        // referenced source name
        String,
        #[snippet(0, message("{}", self.4))] SourceSpan,
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
            src: String,
            #[snippet(src, message("hi this is where the thing went wrong"))]
            snip: SourceSpan,
            #[highlight(snip)]
            var1: SourceSpan,
            #[highlight(snip)]
            var2: SourceSpan,
        },
        #[diagnostic(code(foo::b))]
        B(
            String,
            #[snippet(0, message("hi"))] SourceSpan,
            #[highlight(1)] SourceSpan,
            #[highlight(1, label("var 2"))] SourceSpan,
            // referenced source name
            #[snippet(0)] SourceSpan,
            #[highlight(4)] SourceSpan,
            #[highlight(4)] SourceSpan,
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
    code(foo::bar::baz),
    url("https://example.com"),
    help("help"),
    severity(Warning)
)]
struct ForwardsTo {
    src: String,
    #[snippet(src, message("snippet text"))]
    snip: miette::SourceSpan,
    #[highlight(snip, label("highlight text"))]
    highlight: miette::SourceSpan,
}

impl ForwardsTo {
    fn new() -> Self {
        ForwardsTo {
            src: SNIPPET_TEXT.into(),
            snip: SourceSpan::new(0.into(), SNIPPET_TEXT.len().into()),
            highlight: SourceSpan::new(11.into(), 6.into()),
        }
    }
}

fn check_all(diag: &impl Diagnostic) {
    // check Diagnostic impl forwards all these methods
    assert_eq!(diag.code().to_string(), "foo::bar::baz");
    assert_eq!(diag.url().unwrap().to_string(), "https://example.com");
    assert_eq!(diag.help().unwrap().to_string(), "help");
    assert_eq!(diag.severity().unwrap(), miette::Severity::Warning);
}

fn check_snippets(diag: &impl Diagnostic) {
    type Snip = (Option<String>, usize, usize);
    let snips: Vec<(Snip, Vec<Snip>)> = diag
        .snippets()
        .unwrap()
        .map(
            |miette::DiagnosticSnippet {
                 message,
                 context,
                 highlights,
                 ..
             }| {
                (
                    (message, context.offset(), context.len()),
                    highlights
                        .into_iter()
                        .flatten()
                        .map(|(msg, span)| (msg, span.offset(), span.len()))
                        .collect(),
                )
            },
        )
        .collect();
    assert_eq!(
        &snips[..],
        &[(
            (Some("snippet text".into()), 0, SNIPPET_TEXT.len()),
            vec![(Some("highlight text".into()), 11, 6)]
        )]
    );
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
    assert_eq!(bar_variant.code().to_string(), "foo::bar::bar_variant");
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
    #[error(transparent)]
    #[diagnostic(code(foo::bar::baz), forward(single_field))]
    struct Struct {
        #[from]
        single_field: ForwardsTo,
    }
    // Also check the From impl here
    let variant: Struct = ForwardsTo::new().into();
    check_snippets(&variant);
}
