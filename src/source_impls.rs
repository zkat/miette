/*!
Default trait implementations for [`SourceCode`].
*/
use std::{borrow::Cow, collections::VecDeque, fmt::Debug, sync::Arc};

use crate::{MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};

#[derive(Clone, Copy)]
struct LineInfo {
    line_no: usize,
    start: usize,
    end: usize,
}

fn context_info<'a>(
    input: &'a [u8],
    span: &SourceSpan,
    context_lines_before: Option<usize>,
    context_lines_after: Option<usize>,
) -> Result<MietteSpanContents<'a>, MietteError> {
    let mut iter = input
        .split_inclusive(|b| *b == b'\n')
        .enumerate()
        .map(|(line_no, line)| {
            // SAFETY:
            // - it is safe to use `offset_from` on slices of an array per Rust design (max array size)
            //   (https://doc.rust-lang.org/stable/reference/types/numeric.html#machine-dependent-integer-types)
            // - since `line` is a slice of `input, the offset cannot be negative either
            let offset = unsafe { line.as_ptr().offset_from(input.as_ptr()) } as usize;
            LineInfo {
                line_no,
                start: offset,
                end: offset + line.len(),
            }
        });

    // First line handled separately because otherwise, we can't distinguish
    // between `None` from an empty document (which we still want to handle as
    // a "single empty line"), and `None` from the end of the document.
    let mut line_starts = VecDeque::new();
    let mut line_info = match iter.next() {
        None => LineInfo {
            line_no: 0,
            start: 0,
            end: 0,
        },
        Some(info) => info,
    };
    line_starts.push_back(line_info);

    // Get the "before" lines (including the line containing the start
    // of the span)
    while span.offset() >= line_info.end {
        line_info = match iter.next() {
            None => break,
            Some(info) => info,
        };

        if line_starts.len() > context_lines_before.unwrap_or(0) {
            line_starts.pop_front();
        }
        line_starts.push_back(line_info);
    }
    let (start_lineno, start_offset, start_column) = {
        let start_info = line_starts.pop_front().unwrap();
        if context_lines_before.is_some() {
            (start_info.line_no, start_info.start, 0)
        } else {
            (
                start_info.line_no,
                span.offset(),
                span.offset() - start_info.start,
            )
        }
    };

    // Find the end of the span
    while span.offset() + span.len() > line_info.end {
        line_info = match iter.next() {
            None => break,
            Some(info) => info,
        };
    }

    // Get the "after" lines
    if let Some(last) = iter.take(context_lines_after.unwrap_or(0)).last() {
        line_info = last;
    }
    if span.offset() + span.len() > line_info.end {
        return Err(MietteError::OutOfBounds);
    }
    let (end_lineno, end_offset) = if context_lines_after.is_some() {
        (line_info.line_no, line_info.end)
    } else {
        (line_info.line_no, span.offset() + span.len())
    };

    Ok(MietteSpanContents::new(
        &input[start_offset..end_offset],
        (start_offset..end_offset).into(),
        start_lineno,
        start_column,
        end_lineno - start_lineno + 1,
    ))
}

impl SourceCode for [u8] {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let contents = context_info(self, span, context_lines_before, context_lines_after)?;
        Ok(Box::new(contents))
    }
}

impl<'src> SourceCode for &'src [u8] {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <[u8] as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl SourceCode for Vec<u8> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <[u8] as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl SourceCode for str {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
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
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl SourceCode for String {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(self, span, context_lines_before, context_lines_after)
    }
}

impl<T: ?Sized + SourceCode> SourceCode for Arc<T> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        self.as_ref()
            .read_span(span, context_lines_before, context_lines_after)
    }
}

impl<T: ?Sized + SourceCode + ToOwned> SourceCode for Cow<'_, T>
where
    // The minimal bounds are used here.
    // `T::Owned` need not be
    // `SourceCode`, because `&T`
    // can always be obtained from
    // `Cow<'_, T>`.
    T::Owned: Debug + Send + Sync,
{
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: Option<usize>,
        context_lines_after: Option<usize>,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
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
        let contents = src.read_span(&(0, 4).into(), None, None)?;
        assert_eq!("foo\n", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((0, 4)), *contents.span());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn shifted() -> Result<(), MietteError> {
        let src = String::from("foobar");
        let contents = src.read_span(&(3, 3).into(), Some(1), Some(1))?;
        assert_eq!("foobar", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((0, 6)), *contents.span());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn middle() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(4, 4).into(), None, None)?;
        assert_eq!("bar\n", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((4, 4)), *contents.span());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn middle_of_line() -> Result<(), MietteError> {
        let src = String::from("foo\nbarbar\nbaz\n");
        let contents = src.read_span(&(7, 4).into(), None, None)?;
        assert_eq!("bar\n", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((7, 4)), *contents.span());
        assert_eq!(1, contents.line());
        assert_eq!(3, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn end_of_line_before_newline() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(7, 0).into(), None, None)?;
        assert_eq!("", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((7, 0)), *contents.span());
        assert_eq!(1, contents.line());
        assert_eq!(3, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn end_of_line_after_newline() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(8, 0).into(), None, None)?;
        assert_eq!("", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((8, 0)), *contents.span());
        assert_eq!(2, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn end_of_file_with_newline() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz\n");
        let contents = src.read_span(&(12, 0).into(), None, None)?;
        assert_eq!("", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((12, 0)), *contents.span());
        assert_eq!(2, contents.line());
        assert_eq!(4, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn end_of_file_without_newline() -> Result<(), MietteError> {
        let src = String::from("foo\nbar\nbaz");
        let contents = src.read_span(&(11, 0).into(), None, None)?;
        assert_eq!("", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((11, 0)), *contents.span());
        assert_eq!(2, contents.line());
        assert_eq!(3, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn with_crlf() -> Result<(), MietteError> {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents = src.read_span(&(5, 5).into(), None, None)?;
        assert_eq!("bar\r\n", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((5, 5)), *contents.span());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());
        Ok(())
    }

    #[test]
    fn with_context() -> Result<(), MietteError> {
        let src = String::from("xxx\nfoo\nbar\nbaz\n\nyyy\n");
        let contents = src.read_span(&(8, 3).into(), Some(1), Some(1))?;
        assert_eq!(
            "foo\nbar\nbaz\n",
            std::str::from_utf8(contents.data()).unwrap()
        );
        assert_eq!(SourceSpan::from((4, 12)), *contents.span());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(3, contents.line_count());
        Ok(())
    }

    #[test]
    fn multiline_with_context() -> Result<(), MietteError> {
        let src = String::from("aaa\nxxx\n\nfoo\nbar\nbaz\n\nyyy\nbbb\n");
        let contents = src.read_span(&(9, 11).into(), Some(1), Some(1))?;
        assert_eq!(
            "\nfoo\nbar\nbaz\n\n",
            std::str::from_utf8(contents.data()).unwrap()
        );
        assert_eq!(SourceSpan::from((8, 14)), *contents.span());
        assert_eq!(2, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(5, contents.line_count());
        let span: SourceSpan = (8, 14).into();
        assert_eq!(&span, contents.span());
        Ok(())
    }

    #[test]
    fn multiline_with_context_line_start() -> Result<(), MietteError> {
        let src = String::from("one\ntwo\n\nthree\nfour\nfive\n\nsix\nseven\n");
        let contents = src.read_span(&(2, 0).into(), Some(2), Some(2))?;
        assert_eq!(
            "one\ntwo\n\n",
            std::str::from_utf8(contents.data()).unwrap()
        );
        assert_eq!(SourceSpan::from((0, 9)), *contents.span());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        let span: SourceSpan = (0, 9).into();
        assert_eq!(&span, contents.span());
        assert_eq!(3, contents.line_count());
        Ok(())
    }

    #[test]
    fn empty_source() -> Result<(), MietteError> {
        let src = String::from("");

        let contents = src.read_span(&(0, 0).into(), None, None)?;
        assert_eq!("", std::str::from_utf8(contents.data()).unwrap());
        assert_eq!(SourceSpan::from((0, 0)), *contents.span());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        assert_eq!(1, contents.line_count());

        Ok(())
    }

    #[test]
    fn empty_source_out_of_bounds() {
        let src = String::from("");

        let contents = src.read_span(&(0, 1).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));

        let contents = src.read_span(&(0, 2).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));

        let contents = src.read_span(&(1, 0).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));

        let contents = src.read_span(&(1, 1).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));

        let contents = src.read_span(&(2, 0).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));

        let contents = src.read_span(&(2, 1).into(), None, None);
        assert!(matches!(contents, Err(MietteError::OutOfBounds)));
    }
}
