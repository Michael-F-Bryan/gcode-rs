//! A zero-allocation push-based parser for g-code.
//!
//! # How the API works
//!
//! Parsing is **push-based**: the parser drives the visitor. You implement
//! visitor traits and pass a visitor into [`parse`]; the parser then calls your
//! methods as it encounters each item in the source.
//!
//! **Nested visitors:** The API is structured in layers. A [`ProgramVisitor`]
//! receives [`start_block`](ProgramVisitor::start_block) and returns a
//! [`ControlFlow`] containing a [`BlockVisitor`] for that line. The line
//! visitor receives line numbers, comments, and—when a G/M/O/T command
//! starts—[`start_general_code`](BlockVisitor::start_general_code), which returns a
//! [`ControlFlow`] containing a [`CommandVisitor`] for that command. Each
//! visitor can be a new value that borrows from the parent, so the whole chain
//! can be zero-allocation.
//!
//! **Pausing and resuming:** If a visitor returns [`ControlFlow::Break`], the
//! parser stops (e.g. because a buffer is full). To resume parsing, call
//! [`resume`] again with the same parser and visitor; parsing continues from
//! where it left off. [`ControlFlow::Continue`] supplies the
//! next-level visitor and continues.
//!
//! **Errors:** Recoverable errors are reported via callback methods on the
//! visitor for the level where they occur (e.g. unexpected line number on
//! [`BlockVisitor`], argument overflow on [`CommandVisitor`]). The parser does
//! not abort; it calls the callback and continues.

#![allow(missing_docs)]

mod lexer;
mod parser;
mod types;

pub use self::parser::{ParserState, resume};
pub use self::types::{
    BlockVisitor, CommandVisitor, ControlFlow, Diagnostics, HasDiagnostics,
    Noop, Number, ProgramVisitor, Span, Value,
};

pub fn parse(src: &str, visitor: &mut impl ProgramVisitor) {
    let parser = ParserState::empty();
    let _ = resume(parser, src, visitor);
}
