use crate::{MietteError, SourceCode, SourceSpan, SpanContents};

/// Utility struct for adding attributes such as `name` and `language` to a [`SourceCode`],
/// or if you want to override those attributes from another [`SourceCode`]:
///
/// ```
/// # use miette::MietteSourceCode;
/// let src = MietteSourceCode::new("fn f() {}").with_name("snippet").with_language("Rust");
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MietteSourceCode<S: SourceCode + 'static> {
    source: S,
    name: Option<String>,
    language: Option<String>,
}

impl<S: SourceCode> MietteSourceCode<S> {
    /// Make a new [`MietteSourceCode`] object.
    pub fn new(source: S) -> Self {
        Self {
            source,
            name: None,
            language: None,
        }
    }

    /// Set the name for this source code.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    /// Set the language name for this source code.
    ///
    /// This is used to drive syntax highlighting when the `syntect-highlighter`
    /// feature is enabled.
    ///
    /// Examples: `"Rust"`, `"Python"`, `"C"`
    ///
    /// The list of language names comes from the [`syntect`] crate.
    /// See https://github.com/trishume/syntect/issues/168
    pub fn with_language(mut self, language: impl AsRef<str>) -> Self {
        self.language = Some(language.as_ref().to_string());
        self
    }
}

impl<S: SourceCode> std::fmt::Debug for MietteSourceCode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedSource")
            .field("name", &self.name)
            .field("language", &self.language)
            .field("source", &"<redacted>");
        Ok(())
    }
}

impl<S: SourceCode> SourceCode for MietteSourceCode<S> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        self.source
            .read_span(span, context_lines_before, context_lines_after)
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

impl<S: SourceCode> From<S> for MietteSourceCode<S> {
    fn from(source: S) -> Self {
        MietteSourceCode::new(source)
    }
}
