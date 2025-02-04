use crate::protocol::{LabeledSpan, SourceSpan};

// Huge thanks to @jam1gamer for this hack:
// https://twitter.com/jam1garner/status/1515887996444323840

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct OptionalWrapper<T>(pub core::marker::PhantomData<T>);

#[doc(hidden)]
pub trait ToOption {
    #[doc(hidden)]
    fn to_option<T>(value: T) -> Option<T>;
}

impl<T> OptionalWrapper<Option<T>> {
    #[doc(hidden)]
    pub fn to_option(value: &Option<T>) -> &Option<T> {
        value
    }
}

impl<T> ToOption for OptionalWrapper<T> {
    fn to_option<U>(value: U) -> Option<U> {
        Some(value)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ToLabelSpanWrapper {}
pub trait ToLabeledSpan<T> {
    #[doc(hidden)]
    fn to_labeled_span(span: T) -> LabeledSpan;
}
impl ToLabeledSpan<LabeledSpan> for ToLabelSpanWrapper {
    fn to_labeled_span(span: LabeledSpan) -> LabeledSpan {
        span
    }
}
impl<T> ToLabeledSpan<T> for ToLabelSpanWrapper
where
    T: Into<SourceSpan>,
{
    fn to_labeled_span(span: T) -> LabeledSpan {
        LabeledSpan::new_with_span(None, span.into())
    }
}
