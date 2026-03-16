//! A crate for parsing g-code programs, designed with embedded environments in
//! mind.
//!
//! Some explicit design goals of this crate are:
//!
//! - **embedded-friendly:** users should be able to use this crate without
//!   requiring access to an operating system (e.g. `#[no_std]` environments or
//!   WebAssembly)
//! - **deterministic memory usage:** the library can be tweaked to use no
//!   dynamic allocation (see the [`crate::core`] module)
//! - **error-resistant:** erroneous input won't abort parsing, instead
//!   notifying the caller and continuing on (see [`crate::core::Diagnostics`])
//! - **performance:** parsing should be reasonably fast, guaranteeing `O(n)`
//!   time complexity with no backtracking
//!
//! # Getting Started
//!
//! ## Simple parsing (with `alloc`)
//!
//! With the [`alloc`] feature (enabled by default), use [`parse`] to get a
//! [`Program`] and any [`Diagnostics`]. You can then walk [`Block`]s and
//! inspect [`Code`]s (e.g. [`Code::General`]) and their [`Argument`]s.
//!
//! ```rust,ignore
//! # #[cfg(feature = "alloc")]
//! # fn main() -> Result<(), gcode::Diagnostics> {
//! use gcode::{Code, Value};
//!
//! let src = "G90 (absolute)\nG00 X50.0 Y-10";
//! let result = gcode::parse(src)?;
//!
//! let program = result;
//! assert!(program.blocks.len() >= 1);
//!
//! for block in &program.blocks {
//!     for code in &block.codes {
//!         if let Code::General(g) = code {
//!             let x = g.args.iter().find(|a| a.letter == 'X').map(|a| a.value);
//!             let y = g.args.iter().find(|a| a.letter == 'Y').map(|a| a.value);
//!             if let (Some(Value::Literal(x)), Some(Value::Literal(y))) = (x, y) {
//!                 assert_eq!((x, y), (50.0, -10.0));
//!             }
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Parse errors are collected as [`Diagnostic`]s; if [`Diagnostics::is_empty`]
//! is false, [`parse`] returns `Err(diagnostics)`.
//!
//! ## Push-based / zero-allocation parsing
//!
//! For no-heap use or full control, use the [`core`] module. Implement
//! [`ProgramVisitor`](crate::core::ProgramVisitor): the parser calls
//! [`ProgramVisitor::start_block`](crate::core::ProgramVisitor::start_block) and
//! you return a [`ControlFlow::Continue`](crate::core::ControlFlow) with a
//! [`BlockVisitor`](crate::core::BlockVisitor). That visitor receives
//! [`BlockVisitor::line_number`](crate::core::BlockVisitor::line_number),
//! [`BlockVisitor::comment`](crate::core::BlockVisitor::comment), and
//! [`BlockVisitor::start_general_code`](crate::core::BlockVisitor::start_general_code)
//! (and similar for M/O/T), returning a
//! [`CommandVisitor`](crate::core::CommandVisitor) for each command. See
//! [`crate::core`] for the full visitor model and [`resume`](crate::core::resume)
//! for pause/resume.
//!
//! ```rust
//! use gcode::core::{
//!     BlockVisitor, CommandVisitor, ControlFlow, Noop, ProgramVisitor,
//! };
//!
//! let src = "G90 G01 X5";
//! gcode::core::parse(src, &mut Noop);
//! ```
//!
//! # Zero allocation
//!
//! To avoid dynamic allocation, do not enable the `alloc` feature and do not
//! use the [`parse`] function (which builds an AST). Implement the
//! [`ProgramVisitor`](crate::core::ProgramVisitor),
//! [`BlockVisitor`](crate::core::BlockVisitor), and
//! [`CommandVisitor`](crate::core::CommandVisitor) traits and pass your visitor
//! to [`core::parse`]; the parser drives your visitor and
//! does not allocate.
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
//! # Cargo Features
//!
//! - **alloc** (default): enables the [`ast`] module and [`parse`], producing
//!   a [`Program`] and collecting [`Diagnostics`].
//! - **serde** (default): enables serialisation and deserialisation of core and
//!   AST types via `serde`.
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
    // rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![cfg_attr(not(test), no_std)]
// Make sure docs indicate when something is hidden behind a feature flag
#![cfg_attr(feature = "unstable-doc-cfg", feature(doc_cfg))]
#![cfg_attr(feature = "unstable-doc-cfg", doc(auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod ast;

pub mod core;

#[cfg(feature = "alloc")]
pub use crate::ast::parse;

#[cfg(doc)]
use crate::ast::*;
