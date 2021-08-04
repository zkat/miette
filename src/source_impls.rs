/*!
Default trait implementations for [Source].
*/
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

use utf8_chars::BufReadCharsExt;

use crate::{MietteError, Source, SourceLocation, SourceOffset, SourceSpan};

impl Source for String {
    fn read_span(&self, span: &SourceSpan) -> Result<Vec<u8>, MietteError> {
        Ok(self
            .as_bytes()
            .iter()
            .skip(span.start.bytes())
            .take(span.end.bytes() - span.start.bytes())
            .copied()
            .collect())
    }

    fn find_location(&self, src_offset: SourceOffset) -> Result<SourceLocation, MietteError> {
        let mut offset = 0usize;
        let mut line = 0usize;
        let mut column = 0usize;
        let mut iter = self.chars().peekable();
        while let Some(char) = iter.next() {
            if offset >= src_offset.bytes() {
                return Ok(SourceLocation { line, column });
            }
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                    }
                    line += 1;
                    column = 0;
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    column += 1;
                }
            }
        }
        Ok(SourceLocation { line, column })
    }

    fn find_offset(&self, location: &SourceLocation) -> Result<SourceOffset, MietteError> {
        let mut offset = 0usize;
        let mut line = 0usize;
        let mut column = 0usize;
        let mut iter = self.chars().peekable();
        while let Some(char) = iter.next() {
            if line >= location.line && column >= location.column {
                return Ok(SourceOffset::from(offset));
            }
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                    }
                    line += 1;
                    column = 0;
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    column += 1;
                }
            }
        }
        Ok(SourceOffset::from(offset))
    }
}

impl Source for PathBuf {
    fn read_span(&self, span: &SourceSpan) -> Result<Vec<u8>, MietteError> {
        let file = BufReader::new(File::open(self)?);
        Ok(file
            .bytes()
            .skip(span.start.bytes())
            .take(span.end.bytes() - span.start.bytes())
            .collect::<Result<Vec<u8>, io::Error>>()?)
    }

    fn find_location(&self, src_offset: SourceOffset) -> Result<SourceLocation, MietteError> {
        let mut file = BufReader::new(File::open(self)?);
        let mut offset = 0usize;
        let mut line = 0usize;
        let mut column = 0usize;
        let mut iter = file.chars().peekable();
        while let Some(char) = iter.next() {
            if offset >= src_offset.bytes() {
                return Ok(SourceLocation { line, column });
            }
            let char = char?;
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if(|res| matches!(res, Ok('\n'))).is_some() {
                        offset += 1;
                    }
                    line += 1;
                    column = 0;
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    column += 1;
                }
            }
        }
        Ok(SourceLocation { line, column })
    }

    fn find_offset(&self, location: &SourceLocation) -> Result<SourceOffset, MietteError> {
        let mut file = BufReader::new(File::open(self)?);
        let mut offset = 0usize;
        let mut line = 0usize;
        let mut column = 0usize;
        let mut iter = file.chars().peekable();
        while let Some(char) = iter.next() {
            if line >= location.line && column >= location.column {
                return Ok(SourceOffset::from(offset));
            }
            let char = char?;
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if(|res| matches!(res, Ok('\n'))).is_some() {
                        offset += 1;
                    }
                    line += 1;
                    column = 0;
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    column += 1;
                }
            }
        }
        Ok(SourceOffset::from(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;

    #[test]
    fn basic() -> Result<(), MietteError> {
        let src = String::from("foo\n");
        let span = SourceSpan {
            label: "idk".into(),
            start: src.find_offset(&SourceLocation { line: 0, column: 0 })?,
            end: src.find_offset(&SourceLocation { line: 1, column: 0 })?,
        };
        assert_eq!(
            "foo\n",
            std::str::from_utf8(&src.read_span(&span)?).unwrap()
        );
        Ok(())
    }

    #[test]
    fn middle() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let span = SourceSpan {
            label: "idk".into(),
            start: src.find_offset(&SourceLocation { line: 1, column: 0 })?,
            end: src.find_offset(&SourceLocation { line: 2, column: 0 })?,
        };
        assert_eq!(
            "bar\n",
            std::str::from_utf8(&src.read_span(&span)?).unwrap()
        );
        Ok(())
    }

    #[test]
    fn with_crlf() -> Result<(), MietteError> {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let span = SourceSpan {
            label: "idk".into(),
            start: src.find_offset(&SourceLocation { line: 1, column: 0 })?,
            end: src.find_offset(&SourceLocation { line: 2, column: 0 })?,
        };
        assert_eq!(
            "bar\r\n",
            std::str::from_utf8(&src.read_span(&span)?).unwrap()
        );
        Ok(())
    }
}
