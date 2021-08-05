/*!
Default trait implementations for [Source].
*/
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use utf8_chars::BufReadCharsExt;

use crate::{MietteError, Source, SourceLocation, SourceSpan, SpanContents};

impl Source for String {
    fn read_span(&self, span: &SourceSpan) -> Result<SpanContents, MietteError> {
        let mut offset = 0usize;
        let mut start_line = 0usize;
        let mut start_column = 0usize;
        let mut iter = self.chars().peekable();
        let mut data = Vec::new();
        let mut charbuf = [0; 4];
        while let Some(char) = iter.next() {
            if offset < span.start.bytes() {
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
            } else {
                let len = char.encode_utf8(&mut charbuf).len();
                for byte in &charbuf[0..len] {
                    data.push(*byte);
                }
            }

            if offset >= span.end.bytes() {
                return Ok(SpanContents::new(
                    data,
                    SourceLocation {
                        line: start_line,
                        column: start_column,
                    },
                ));
            }

            offset += char.len_utf8();
        }
        Err(MietteError::OutOfBounds)
    }
}

impl Source for PathBuf {
    fn read_span(&self, span: &SourceSpan) -> Result<SpanContents, MietteError> {
        let mut buf = BufReader::new(File::open(&self)?);
        let mut offset = 0usize;
        let mut start_line = 0usize;
        let mut start_column = 0usize;
        let mut iter = buf.chars().peekable();
        let mut data = Vec::new();
        let mut charbuf = [0; 4];
        while let Some(char) = iter.next() {
            let char = char?;
            if offset < span.start.bytes() {
                match char {
                    '\r' => {
                        if iter.next_if(|c| matches!(c, Ok('\n'))).is_some() {
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
            } else {
                let len = char.encode_utf8(&mut charbuf).len();
                for byte in &charbuf[0..len] {
                    data.push(*byte);
                }
            }

            if offset >= span.end.bytes() {
                return Ok(SpanContents::new(
                    data,
                    SourceLocation {
                        line: start_line,
                        column: start_column,
                    },
                ));
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
