use core::cmp;
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
    /// The (zero-based) line number.
    pub line: usize,
}

impl Span {
    /// Get the string this [`Span`] corresponds to.
    ///
    /// Passing in a different string will probably lead to... strange...
    /// results.
    pub fn get_text<'input>(&self, src: &'input str) -> Option<&'input str> {
        src.get(self.start..self.end)
    }

    /// Merge two [`Span`]s.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
            line: cmp::min(self.line, other.line),
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(other: Span) -> Range<usize> {
        other.start..other.end
    }
}
