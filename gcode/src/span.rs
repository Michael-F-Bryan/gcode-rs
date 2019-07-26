use core::cmp;
use core::ops::Range;

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    /// A placeholder [`Span`] which will be ignored by [`Span::merge()`].
    pub const PLACEHOLDER: Span =
        Span::new(usize::max_value(), usize::max_value(), usize::max_value());

    pub const fn new(start: usize, end: usize, line: usize) -> Self {
        Span { start, end, line }
    }

    /// Get the string this [`Span`] corresponds to.
    ///
    /// Passing in a different string will probably lead to... strange...
    /// results.
    pub fn get_text<'input>(&self, src: &'input str) -> Option<&'input str> {
        src.get(self.start..self.end)
    }

    /// Merge two [`Span`]s, making sure [`Span::PLACEHOLDER`] spans go away.
    pub fn merge(self, other: Span) -> Span {
        if self == Span::PLACEHOLDER {
            other
        } else if other == Span::PLACEHOLDER {
            self
        } else {
            Span {
                start: cmp::min(self.start, other.start),
                end: cmp::max(self.end, other.end),
                line: cmp::min(self.line, other.line),
            }
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(other: Span) -> Range<usize> {
        other.start..other.end
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::PLACEHOLDER
    }
}
