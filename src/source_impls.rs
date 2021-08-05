/*!
Default trait implementations for [Source].
*/
use crate::{MietteError, MietteSpanContents, Source, SourceSpan, SpanContents};

impl Source for String {
    fn read_span(&self, span: &SourceSpan) -> Result<Box<dyn SpanContents + '_>, MietteError> {
        let mut offset = 0usize;
        let mut start_line = 0usize;
        let mut start_column = 0usize;
        let mut iter = self.chars().peekable();
        while let Some(char) = iter.next() {
            if offset < span.start.offset() {
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

            if offset >= span.end.offset() {
                return Ok(Box::new(MietteSpanContents::new(
                    &self.as_bytes()[span.start.offset()..=span.end.offset()],
                    start_line,
                    start_column,
                )));
            }

            offset += char.len_utf8();
        }
        Err(MietteError::OutOfBounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<(), MietteError> {
        let src = String::from("foo\n");
        let contents = src.read_span(&SourceSpan {
            start: 0.into(),
            end: 3.into(),
        })?;
        assert_eq!("foo\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn middle() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&SourceSpan {
            start: 4.into(),
            end: 7.into(),
        })?;
        assert_eq!("bar\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }

    #[test]
    fn with_crlf() -> Result<(), MietteError> {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents = src.read_span(&SourceSpan {
            start: 5.into(),
            end: 9.into(),
        })?;
        assert_eq!("bar\r\n", std::str::from_utf8(contents.data()).unwrap());
        Ok(())
    }
}
