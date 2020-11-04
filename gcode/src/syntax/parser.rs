use crate::Span;
use core::fmt::{self, Display, Formatter};

/// An incremental push parser for the G-Code language.
#[derive(Debug)]
pub struct Parser {
    state: ParserState,
}

impl Parser {
    pub const fn new() -> Self {
        Parser {
            state: ParserState {
                current_line: None,
                last_g_code: None,
            },
        }
    }

    /// Process another chunk of input, triggering methods on the [`Callbacks`]
    /// to notify the caller when things are encountered.
    ///
    /// # Notes
    ///
    /// The [`Parser`] is intended to work with chunks of input and will assume
    /// it is starting *exactly* where it left off.
    pub fn process<C>(
        &mut self,
        chunk: &str,
        callbacks: &mut C,
    ) -> Result<(), ParseError<C::Error>>
    where
        C: Callbacks,
    {
        for token in super::tokenize(chunk) {
            todo!()
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ParserState {
    pub current_line: Option<usize>,
    pub last_g_code: Option<f64>,
}

/// Callbacks fired by the [`Parser`] during the parsing process.
pub trait Callbacks {
    type Error;

    fn on_begin_line(
        &mut self,
        _span: Span,
        _line: &str,
        _skipped: bool,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_end_line(
        &mut self,
        _span: Span,
        _line: &str,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_word(
        &mut self,
        _span: Span,
        _word: Word,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_comment(
        &mut self,
        _span: Span,
        comment: Comment<'_>,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_consume(
        &mut self,
        _span: Span,
        _text: &str,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_percent(
        &mut self,
        _span: Span,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }

    fn on_unknown(
        &mut self,
        _span: Span,
        _unknown_content: &str,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        Continuation::Continue
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Word {
    pub letter: char,
    pub value: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Comment<'a> {
    pub text: &'a str,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Continuation<E> {
    /// Keep going.
    Continue,
    /// Stop parsing gracefully (e.g. because a buffer is full).
    Break,
    /// Consume to the end of the line or the next comment.
    Consume,
    /// Abort with an error.
    Error(E),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParseError<E> {
    pub location: Span,
    pub error: E,
}

impl<E> Display for ParseError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "A parsing error occurred at {}", self.location)
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for ParseError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
