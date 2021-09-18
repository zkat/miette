/*!
Default trait implementations for [SourceCode].
*/
use std::{
    borrow::{Cow, ToOwned},
    fmt::Debug,
    sync::Arc,
};

use crate::{MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};

fn context_info<'a>(
    input: &'a [u8],
    span: &SourceSpan,
    context_lines_before: usize,
    context_lines_after: usize,
) -> Result<(&'a [u8], usize, usize), MietteError> {
    let mut offset = 0usize;
    let mut start_line = 0usize;
    // We don't mess with this for now -- context always assume it starts at
    // the beginning of a line.
    let start_column = 0usize;
    let mut before_lines_starts = Vec::new();
    let mut current_line_start = 0usize;
    let mut end_lines = 0usize;
    let mut post_span = false;
    let mut iter = input.iter().copied().peekable();
    while let Some(char) = iter.next() {
        if matches!(char, b'\r' | b'\n') {
            if char == b'\r' && iter.next_if_eq(&b'\n').is_some() {
                offset += 1;
            }
            if offset < span.offset() {
                // We're before the start of the span.
                start_line += 1;
                before_lines_starts.push(current_line_start);
                if before_lines_starts.len() > context_lines_before {
                    before_lines_starts.remove(0);
                }
            } else if offset >= span.offset() + span.len() - 1 {
                // We're after the end of the span, but haven't necessarily
                // started collecting end lines yet (we might still be
                // collecting context lines).
                if post_span {
                    end_lines += 1;
                    if end_lines > context_lines_after {
                        offset += 1;
                        break;
                    }
                }
            }
            current_line_start = offset + 1;
        }

        if offset >= span.offset() + span.len() - 1 {
            post_span = true;
            if end_lines >= context_lines_after {
                offset += 1;
                break;
            }
        }

        offset += 1;
    }
    if offset >= span.offset() + span.len() - 1 {
        Ok((
            &input[before_lines_starts
                .get(0)
                .copied()
                .unwrap_or_else(|| span.offset())..offset],
            start_line,
            start_column,
        ))
    } else {
        Err(MietteError::OutOfBounds)
    }
}

impl SourceCode for [u8] {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        let (data, start_line, start_column) =
            context_info(self, span, context_lines_before, context_lines_after)?;
        return Ok(Box::new(MietteSpanContents::new(
            data,
            start_line,
            start_column,
        )));
    }
}

impl SourceCode for str {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        <[u8] as SourceCode>::read_span(
            self.as_bytes(),
            span,
            context_lines_before,
            context_lines_after,
        )
    }
}

/// Makes `src: &'static str` or `struct S<'a> { src: &'a str }` usable.
impl<'s> SourceCode for &'s str {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        <str as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl SourceCode for String {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        <str as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl<T: SourceCode> SourceCode for Arc<T> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        self.as_ref()
            .read_span(span, context_lines_before, context_lines_after)
    }
}

impl<T: ?Sized + SourceCode + ToOwned> SourceCode for Cow<'_, T>
where
    // The minimal bounds are used here. `T::Owned` need not be `Source`, because `&T` can always
    // be obtained from `Cow<'_, T>`.
    T::Owned: Debug + Send + Sync,
{
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        self.as_ref()
            .read_span(span, context_lines_before, context_lines_after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<(), MietteError> {
        let src = String::from("foo\n");
        let contents = src.read_span(&(0, 4).into(), 0, 0)?;
        assert_eq!("foo\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn middle() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(4, 4).into(), 0, 0)?;
        assert_eq!("bar\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn with_crlf() -> Result<(), MietteError> {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents = src.read_span(&(5, 5).into(), 0, 0)?;
        assert_eq!("bar\r\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn with_context() -> Result<(), MietteError> {
        let src = String::from("xxx\nfoo\nbar\nbaz\n\nyyy\n");
        let contents = src.read_span(&(10, 4).into(), 1, 2)?;
        assert_eq!(
            "foo\nbar\nbaz\n\n",
            std::str::from_utf8(contents.data()).unwrap()
        );
        Ok(())
    }
}
