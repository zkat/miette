/*!
Default trait implementations for [Source].
*/
use std::{borrow::Cow, sync::Arc};

use crate::{MietteError, MietteSpanContents, Source, SourceSpan, SpanContents};

fn start_line_column(string: &str, span: &SourceSpan) -> Result<(usize, usize), MietteError> {
    let mut offset = 0usize;
    let mut start_line = 0usize;
    let mut start_column = 0usize;
    let mut iter = string.chars().peekable();
    while let Some(char) = iter.next() {
        if offset < span.offset() {
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                    }
                    start_line += 1;
                    start_column = 0;
                }
                '\n' => {
                    start_line += 1;
                    start_column = 0;
                }
                _ => {
                    start_column += 1;
                }
            }
        }

        if offset >= span.offset() + span.len() - 1 {
            return Ok((start_line, start_column));
        }

        offset += char.len_utf8();
    }
    Err(MietteError::OutOfBounds)
}

// The basic impl here is on str (not &str), because otherwise String's impl cannot reuse it
// without creating a temporary &str inside its read_span implementation, and then returning data
// that refers to that temporary.
impl Source for str {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        let (start_line, start_column) = start_line_column(self, span)?;
        return Ok(Box::new(MietteSpanContents::new(
            &self.as_bytes()[span.offset()..span.offset() + span.len()],
            start_line,
            start_column,
        )));
    }
}

/// Makes `src: &'static str` or `struct S<'a> { src: &'a str }` usable.
impl<'s> Source for &'s str {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        <str as Source>::read_span(self, span)
    }
}

impl Source for String {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        <str as Source>::read_span(self, span)
    }
}

impl<T: Source> Source for Arc<T> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        self.as_ref().read_span(span)
    }
}

impl<T: Source + Clone> Source for Cow<'_, T> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
    ) -> Result<Box<dyn SpanContents + 'a>, MietteError> {
        self.as_ref().read_span(span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<(), MietteError> {
        let src = String::from("foo\n");
        let contents = src.read_span(&(0, 4).into())?;
        assert_eq!("foo\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn middle() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(4, 4).into())?;
        assert_eq!("bar\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn with_crlf() -> Result<(), MietteError> {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents = src.read_span(&(5, 5).into())?;
        assert_eq!("bar\r\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }
}
