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

mod lexer;
mod parser;
mod types;

pub use self::parser::{ParserState, resume};
pub use self::types::{
    CommandVisitor, ControlFlow, LineVisitor, Number, ProgramVisitor, Span,
};

pub fn parse(src: &str, visitor: impl ProgramVisitor) {
    let parser = ParserState::empty();
    let _ = resume(parser, src, visitor);
}
