//! A zero-allocation push-based parser for g-code.
//!
//! # How the API works
//!
//! Parsing is **push-based**: the parser drives the visitor. You implement
//! visitor traits and pass a visitor into [`parse`]; the parser then calls your
//! methods as it encounters each item in the source.
//!
//! **Nested visitors:** The API is structured in layers. A [`ProgramVisitor`]
//! receives [`start_line`](ProgramVisitor::start_line) and returns a
//! [`ControlFlow`] containing a [`LineVisitor`] for that line. The line
//! visitor receives line numbers, comments, and—when a G/M/O/T command
//! starts—[`start_g_code`](LineVisitor::start_g_code), which returns a
//! [`ControlFlow`] containing a [`GCodeVisitor`] for that command. Each
//! visitor can be a new value that borrows from the parent, so the whole chain
//! can be zero-allocation.
//!
//! **Pausing and resuming:** If a visitor returns [`ControlFlow::Break`], the
//! parser stops (e.g. because a buffer is full). To resume, call
//! [`Parser::parse`] again with the same parser and visitor; parsing
//! continues from where it left off. [`ControlFlow::Skip`] skips the current
//! item without stopping. [`ControlFlow::Continue(visitor)`] supplies the
//! next-level visitor and continues.
//!
//! **Errors:** Recoverable errors are reported via callback methods on the
//! visitor for the level where they occur (e.g. unexpected line number on
//! [`LineVisitor`], argument overflow on [`GCodeVisitor`]). The parser does
//! not abort; it calls the callback and continues.

#![allow(missing_docs)]

use core::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Number {
    pub major: u16,
    pub minor: Option<u16>,
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Number { major, minor } = self;
        write!(f, "{}", major)?;
        if let Some(minor) = minor {
            write!(f, ".{}", minor)?;
        }
        Ok(())
    }
}

pub fn parse(src: &str, visitor: impl ProgramVisitor) {
    let mut parser = Parser::new(src);
    parser.parse(visitor);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlFlow<T> {
    Continue(T),
    Break,
}

pub trait ProgramVisitor {
    fn start_line(&mut self, span: Span) -> ControlFlow<impl LineVisitor + '_>;
}

pub trait LineVisitor {
    fn line_number(&mut self, _n: f32, _span: Span) {}
    fn comment(&mut self, _value: &str, _span: Span) {}
    fn program_number(&mut self, _number: Number, _span: Span) {}

    fn start_general_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_miscellaneous_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_tool_change_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }

    fn unknown_content_error(&mut self, _text: &str, _span: Span) {}
    fn unexpected_line_number_error(&mut self, _n: f32, _span: Span) {}
    fn letter_without_number_error(&mut self, _value: &str, _span: Span) {}
    fn number_without_letter_error(&mut self, _value: &str, _span: Span) {}
}

pub trait CommandVisitor {
    fn argument(&mut self, _letter: char, _value: f32, _span: Span) {}

    fn argument_buffer_overflow_error(
        &mut self,
        _letter: char,
        _value: f32,
        _span: Span,
    ) {
    }
}

struct Noop;

impl CommandVisitor for Noop {}

impl LineVisitor for Noop {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parser<'src> {
    src: &'src str,
    current_index: usize,
}

impl<'src> Parser<'src> {
    pub const fn new(src: &'src str) -> Self {
        Self {
            src,
            current_index: 0,
        }
    }

    pub const fn finished(&self) -> bool {
        self.src.is_empty()
    }

    pub fn parse(&mut self, mut _visitor: impl ProgramVisitor) {
        todo!();
    }
}

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
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
    pub const fn new(start: usize, end: usize, line: usize) -> Self {
        assert!(start <= end);
        Self { start, end, line }
    }
}
