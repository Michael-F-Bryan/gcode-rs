//! A parser for g-code programs, with both a convenient AST API and a
//! zero-allocation visitor-based core designed for embedded environments.
//!
//! Some explicit design goals of this crate are:
//!
//! - **embedded-friendly:** users should be able to use this crate without
//!   requiring access to an operating system (e.g. `#[no_std]` environments or
//!   WebAssembly)
//! - **deterministic memory usage:** the [`crate::core`] parser can operate
//!   without dynamic allocation
//! - **error-resistant:** the parser attempts to recover from erroneous input,
//!   reporting diagnostics and continuing where possible
//! - **performance:** parsing should be reasonably fast, guaranteeing `O(n)`
//!   time complexity with no backtracking
//!
//! # Getting Started
//!
//! With the [`alloc`] feature (enabled by default), use [`parse`] to parse
//! g-code into a [`Program`]. If any parse errors are emitted, [`parse`]
//! returns `Err` with the collected [`Diagnostics`].
//!
//! ```rust
//! # #[cfg(feature = "alloc")]
//! # fn main() -> Result<(), gcode::Diagnostics> {
//! use gcode::{Code, Value};
//!
//! let src = "G90 (absolute)\nG00 X50.0 Y-10";
//! let program = gcode::parse(src)?;
//!
//! assert!(program.blocks.len() >= 1);
//!
//! for block in &program.blocks {
//!     for code in &block.codes {
//!         if let Code::General(g) = code {
//!             for arg in &g.args {
//!                 match (arg.letter, &arg.value) {
//!                     ('X', Value::Literal(x)) => assert_eq!(*x, 50.0),
//!                     ('Y', Value::Literal(y)) => assert_eq!(*y, -10.0),
//!                     _ => {}
//!                 }
//!             }
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```
//!
//! Parse errors are reported as [`Diagnostic`]s and collected in [`Diagnostics`].
//!
//! For more complex use cases, including zero-allocation or streaming parsing,
//! refer to the [`core`] module.
//!
//! # Document model
//!
//! G-code is modelled as a sequence of *blocks*. A [`Program`] (from [`parse`])
//! is the root: it has `blocks`. Each [`Block`] corresponds roughly to one line
//! of source and contains an optional line number (N), comments, G/M/T
//! commands ([`Code`] with [`Argument`]s), and bare word addresses
//! ([`Block::word_addresses`]—e.g. `X10.5` without a preceding G/M/T).
//!
//! For example, this source:
//!
//! ```text
//! G1 X10.5 Y20.0 F1500
//! M3 S1000
//! ; start cutting
//! ```
//!
//! is a document of three blocks: the first has one G-code (G1) with arguments
//! X, Y, F; the second has one M-code (M3) with S; the third has a comment.
//!
//! Unlike JSON or XML, g-code has no single universal grammar; controllers and
//! dialects differ, and the meaning of a block often depends on machine state
//! or dialect rules. This crate therefore models g-code at the *syntactic*
//! level: the parser represents what was written, not what the machine would
//! do. Higher-level interpretation (e.g. whether X/Y are coordinates for a
//! move) is left to downstream code.
//!
//! # Zero allocation
//!
//! To avoid dynamic allocation, do not enable the `alloc` feature and do not
//! use the [`parse`] function (which builds an AST). Implement
//! [`ProgramVisitor`](crate::core::ProgramVisitor),
//! [`BlockVisitor`](crate::core::BlockVisitor), and
//! [`CommandVisitor`](crate::core::CommandVisitor) and pass your visitor to
//! [`core::parse`]; the parser drives your visitor and does not allocate.
//!
//! # Spans
//!
//! Each element's original location in the source is retained as a
//! [`Span`](crate::core::Span).
//!
//! This supports:
//!
//! - Showing where a parsing or semantic error occurred
//! - Highlighting the current command when stepping through a program
//! - Reporting progress (e.g. line/column) to the user or machine
//!
//! In the core API, visitor methods receive a `Span` (e.g.
//! [`BlockVisitor::line_number`](crate::core::BlockVisitor::line_number) and
//! [`BlockVisitor::comment`](crate::core::BlockVisitor::comment)). AST types
//! (with `alloc`) have a `span` field (e.g. [`Block::span`], [`Comment::span`],
//! [`GeneralCode::span`], [`Argument::span`]).
//!
//! # Feature Flags
//!
#![doc = document_features::document_features!()]
#![deny(
    bare_trait_objects,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unsafe_code,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![cfg_attr(not(test), no_std)]
// Make sure docs indicate when something is hidden behind a feature flag
#![cfg_attr(feature = "unstable-doc-cfg", feature(doc_cfg))]
#![cfg_attr(feature = "unstable-doc-cfg", doc(auto_cfg))]

pub mod core;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod diags;
#[cfg(feature = "alloc")]
mod types;
#[cfg(feature = "alloc")]
mod visitor;

#[cfg(feature = "alloc")]
pub use crate::{
    diags::{Diagnostic, DiagnosticKind, Diagnostics},
    types::*,
    visitor::AstBuilder,
};

/// Parse G-code source into a [`Program`] or return [`Diagnostics`] on error.
///
/// Requires the `alloc` feature. For zero-allocation or streaming parsing, use
/// [`core::parse`] with a custom [`ProgramVisitor`](crate::core::ProgramVisitor).
#[cfg(feature = "alloc")]
pub fn parse(src: &str) -> Result<Program, Diagnostics> {
    let mut visitor = AstBuilder::new();
    core::parse(src, &mut visitor);
    visitor.finish()
}

#[doc = include_str!("../README.md")]
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub fn _assert_readme_code_examples_compile() {}
