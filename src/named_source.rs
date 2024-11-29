use crate::{MietteError, SourceCode, SpanContents};

/// Utility struct for when you have a regular [`SourceCode`] type that doesn't
/// implement `name`. For example [`String`]. Or if you want to override the
/// `name` returned by the `SourceCode`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamedSource<S: SourceCode + 'static> {
    source: S,
    name: String,
    language: Option<String>,
}

impl<S: SourceCode> std::fmt::Debug for NamedSource<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedSource")
            .field("name", &self.name)
            .field("source", &"<redacted>")
            .field("language", &self.language);
        Ok(())
    }
}

impl<S: SourceCode + 'static> NamedSource<S> {
    /// Create a new `NamedSource` using a regular [`SourceCode`] and giving
    /// its returned [`SpanContents`] a name.
    pub fn new(name: impl AsRef<str>, source: S) -> Self
    where
        S: Send + Sync,
    {
        Self {
            source,
            name: name.as_ref().to_string(),
            language: None,
        }
    }

    /// Gets the name of this `NamedSource`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference the inner [`SourceCode`] type for this
    /// `NamedSource`.
    pub fn inner(&self) -> &S {
        &self.source
    }

    /// Sets the [`language`](SpanContents::language) for this source code.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}
/// Utility struct used by [`NamedSource`] to attach a file name to an inner [`SpanContents`] value
#[derive(Debug)]
pub struct NamedSpanContents<T: ?Sized> {
    inner: Box<T>,
    name: Box<str>,
    language: Option<Box<str>>,
}
impl<T: SpanContents + ?Sized> SpanContents for NamedSpanContents<T> {
    #[inline]
    fn data(&self) -> &[u8] {
        self.inner.data()
    }
    #[inline]
    fn span(&self) -> &crate::SourceSpan {
        self.inner.span()
    }
    #[inline]
    fn line(&self) -> usize {
        self.inner.line()
    }
    #[inline]
    fn column(&self) -> usize {
        self.inner.column()
    }
    #[inline]
    fn line_count(&self) -> usize {
        self.inner.line_count()
    }
    #[inline]
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    #[inline]
    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

impl<S: SourceCode + 'static> SourceCode for NamedSource<S> {
    fn read_span<'a>(
        &'a self,
        span: &crate::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        let inner_contents =
            self.inner()
                .read_span(span, context_lines_before, context_lines_after)?;
        Ok(Box::new(NamedSpanContents {
            inner: inner_contents,
            name: self.name.clone().into_boxed_str(),
            language: self.language.as_ref().map(|v| v.clone().into_boxed_str()),
        }))
    }
}
