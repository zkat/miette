use crate::Source;

/// Utility struct for when you have a regular [Source] type, such as a String,
/// that doesn't implement `name`, or if you want to override the `.name()`
/// returned by the `Source`.
#[derive(Debug)]
pub struct NamedSource {
    source: Box<dyn Source + Send + Sync + 'static>,
    name: String,
}

impl NamedSource {
    /// Create a new [NamedSource] using a regular [Source] and giving it a [Source::name].
    pub fn new(name: impl AsRef<str>, source: impl Source + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
            name: name.as_ref().to_string(),
        }
    }

    /// Returns a reference the inner [Source] type for this [NamedSource].
    pub fn inner(&self) -> &(dyn Source + Send + Sync + 'static) {
        &*self.source
    }
}

impl Source for NamedSource {
    fn read_span<'a>(
        &'a self,
        span: &crate::SourceSpan,
    ) -> Result<Box<dyn crate::SpanContents + 'a>, crate::MietteError> {
        self.source.read_span(span)
    }

    fn name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}
