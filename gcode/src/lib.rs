mod lexer;

use core::ops::Range;

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Span {
    /// The byte index corresponding to the item's start.
    pub start: usize,
    /// The index one byte past the item's end.
    pub end: usize,
    pub line: usize,
}

impl Span {
    pub fn get_text<'input>(&self, src: &'input str) -> Option<&'input str> {
        src.get(self.start..self.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(other: Span) -> Range<usize> {
        other.start .. other.end
    }
}