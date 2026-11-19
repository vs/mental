//! Where a record or a diagnostic stands in a file: the whole lines a byte
//! span of the file's text covers.

use std::ops::Range;

use mental_model::{Anchor, Id, Lines, Location};

/// The lines a byte span of `text` stands on.
pub(crate) fn lines(text: &str, span: Range<usize>) -> Lines {
    let line = |offset: usize| {
        let before = &text.as_bytes()[..offset.min(text.len())];
        let number = before.split(|byte| *byte == b'\n').count();
        u32::try_from(number).expect("a file mentalc reads has fewer than 2^32 lines")
    };
    let last_byte = span.end.saturating_sub(1).max(span.start);
    Lines::new(line(span.start), line(last_byte))
        .expect("a span's last byte is not before its first")
}

/// A record's anchor: the whole lines its span stands on.
pub(crate) fn anchor(id: Id, path: &str, text: &str, span: Range<usize>) -> Anchor {
    let lines = lines(text, span);
    let [first, last]: [u32; 2] = lines.into();
    let quote: Vec<&str> = text
        .lines()
        .skip(first as usize - 1)
        .take((last - first + 1) as usize)
        .collect();
    Anchor {
        id,
        path: path.to_owned(),
        lines,
        quote: quote.join("\n"),
    }
}

pub(crate) fn location(path: &str, text: &str, span: Range<usize>) -> Location {
    Location {
        path: path.to_owned(),
        lines: lines(text, span),
    }
}
