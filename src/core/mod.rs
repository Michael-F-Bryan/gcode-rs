//! A zero-allocation, push-based parser for g-code.
//!
//! `gcode::core` is the low-level engine of this crate. Instead of building an
//! abstract syntax tree (AST) for you, it drives your own visitor
//! implementations and never allocates on your behalf. Use this module when
//! you need deterministic memory usage (e.g. embedded or `no_std`), when you
//! want to stream g-code and act on it as it arrives, or when you want full
//! control over how commands, numbers, and diagnostics are represented. If you
//! prefer a ready-made AST and default diagnostics, use [`crate::parse`]
//! and the [`crate::Program`] type instead; they are thin layers built on top
//! of [`crate::core`].
//!
//! ## How the API works
//!
//! The parser is built around a **visitor pattern** that mirrors the g-code
//! grammar: the parser drives the visitor, and the visitor trait hierarchy
//! matches the structure of the language. At the top level you implement
//! [`ProgramVisitor`], which creates a [`BlockVisitor`] for each block (line);
//! each block can then create a [`CommandVisitor`] for each command on that
//! line.
//!
//! ### Terminals vs non-terminals
//!
//! The grammar is reflected in the visitor API:
//!
//! - **Terminals** (leaf tokens) are reported by a single method call with the
//!   value and its [`Span`]. No new visitor is created. Examples:
//!   [`BlockVisitor::line_number`], [`BlockVisitor::comment`],
//!   [`BlockVisitor::program_number`], [`CommandVisitor::argument`].
//!
//! - **Non-terminals** (sub-structures) are entered by a method that returns a
//!   **child visitor** inside [`ControlFlow::Continue`]. The parser then drives
//!   that visitor until the sub-structure ends, at which point it calls a
//!   consuming method (`end_line` or `end_command`) and returns to the parent.
//!   Examples: [`ProgramVisitor::start_block`] returns a [`BlockVisitor`] for
//!   that line; [`BlockVisitor::start_general_code`] (and the other
//!   `start_*_code` methods) return a [`CommandVisitor`] for that command.
//!
//! In other words, the call flow is:
//!
//! - parser calls `start_block()` and gets a [`BlockVisitor`]
//! - block visitor receives `line_number`, `comment`, and `start_*_code(…)`
//! - each `start_*_code(…)` returns a [`CommandVisitor`]
//! - command visitor receives one or more `argument(…)` calls and then
//!   `end_command`
//! - control returns to the block visitor for more calls or `end_line`
//! - control returns to the program visitor
//!
//! The type of each level is fixed by the trait; the parser never allocates
//! intermediate nodes.
//!
//! ## Control flow and allocation
//!
//! Because each non-terminal is “enter by returning a visitor, exit by
//! consuming it”, the **caller** can implement visitors as values that only
//! borrow from their parent (for example a struct holding `&mut Vec<Block>`,
//! `&mut Diagnostics`, or other state). The parser only stores and invokes
//! whatever visitor type you supply; it does not build its own trees or
//! buffers. The entire parse can therefore be zero-allocation: no boxes, no
//! `Vec`s, no strings owned by the parser. Allocation happens only if your
//! visitor implementation chooses to allocate (for example to build an AST).
//!
//! If a visitor returns [`ControlFlow::Break`], the parser stops at a
//! well-defined pause point (for example when an output buffer is full). You
//! can resume later by calling [`resume`] with the returned [`ParserState`]
//! and the same visitor, and parsing will continue from where it left off.
//!
//! ## Diagnostics
//!
//! Recoverable diagnostics are reported via [`HasDiagnostics`]: the visitor
//! supplies a [`Diagnostics`] implementation, and the parser calls into it
//! (for example `emit_unknown_content`, `emit_unexpected`) when it encounters
//! bad input. The parser does not abort on these conditions; it reports and
//! continues, so callers can decide how to surface or aggregate errors.
//!
//! ## Relationship to the rest of the crate
//!
//! The higher-level [`crate`] module and the [`crate::parse`] convenience
//! function are implemented in terms of `gcode::core`: they provide visitors
//! that build an owned AST and collect diagnostics into a single value. As a
//! result, the behaviour of the entire crate is defined here; understanding
//! `gcode::core` gives you a precise mental model for how parsing, spans, and
//! diagnostics behave at every layer.

mod lexer;
mod parser;
mod types;

pub use self::parser::{ParserState, resume};
pub use self::types::{
    BlockVisitor, CommandVisitor, ControlFlow, Diagnostics, HasDiagnostics,
    Noop, Number, ProgramVisitor, Span, TokenType, Value,
};

/// Parses `src` from start to finish, driving `visitor` for each block.
///
/// For incremental or resumable parsing, use [`ParserState::empty`] and
/// [`resume`] instead; return [`ControlFlow::Break`] from your visitor when you
/// need to pause, then call [`resume`] with the returned state.
pub fn parse(src: &str, visitor: &mut impl ProgramVisitor) {
    let parser = ParserState::empty();
    let _ = resume(parser, src, visitor);
}
