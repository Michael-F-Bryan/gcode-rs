use core::{
    cmp,
    fmt::{self, Debug, Formatter},
    ops::Range,
};

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Copy, Clone, Eq)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
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
    /// A placeholder [`Span`] which will be ignored by [`Span::merge()`] and
    /// equality checks.
    pub const PLACEHOLDER: Span =
        Span::new(usize::max_value(), usize::max_value(), usize::max_value());

    /// Create a new [`Span`].
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
        if self.is_placeholder() {
            other
        } else if other.is_placeholder() {
            self
        } else {
            Span {
                start: cmp::min(self.start, other.start),
                end: cmp::max(self.end, other.end),
                line: cmp::min(self.line, other.line),
            }
        }
    }

    /// Is this a [`Span::PLACEHOLDER`]?
    pub fn is_placeholder(self) -> bool {
        let Span { start, end, line } = Span::PLACEHOLDER;

        self.start == start && self.end == end && self.line == line
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Span) -> bool {
        let Span { start, end, line } = *other;

        self.is_placeholder()
            || other.is_placeholder()
            || (self.start == start && self.end == end && self.line == line)
    }
}

impl From<Span> for Range<usize> {
    fn from(other: Span) -> Range<usize> { other.start..other.end }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_placeholder() {
            write!(f, "<placeholder>")
        } else {
            let Span { start, end, line } = self;

            f.debug_struct("Span")
                .field("start", start)
                .field("end", end)
                .field("line", line)
                .finish()
        }
    }
}

impl Default for Span {
    fn default() -> Span { Span::PLACEHOLDER }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_is_equal_to_itself() {
        let span = Span::new(1, 2, 3);

        assert_eq!(span, span);
    }

    #[test]
    fn all_spans_are_equal_to_the_placeholder() {
        let inputs = vec![
            Span::default(),
            Span::PLACEHOLDER,
            Span::new(42, 0, 0),
            Span::new(0, 42, 0),
            Span::new(0, 0, 42),
        ];

        for input in inputs {
            assert_eq!(input, Span::PLACEHOLDER);
        }
    }
}
