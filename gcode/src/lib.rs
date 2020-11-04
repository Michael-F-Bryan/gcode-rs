#![cfg_attr(not(feature = "std"), no_std)]

pub mod syntax;

use core::fmt::{self, Display, Formatter};

/// The location of something inside a body of text.
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self { Span { start, end } }

    pub const fn len(self) -> usize { self.end - self.start }

    pub fn lookup(self, original_text: &str) -> Option<&str> {
        original_text.get(self.start..self.end)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
