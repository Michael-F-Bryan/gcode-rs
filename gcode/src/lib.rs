//! A crate for parsing g-code programs, designed with embedded environments in
//! mind.
//!
//! Some explicit design goals of this crate are:
//!
//! - **embedded-friendly:** users should be able to use this crate without
//!   requiring access to an operating system (e.g. `#[no_std]` environments or
//!   WebAssembly)
//! - **deterministic memory usage:** the library can be tweaked to use no
//!   dynamic allocation (see [`buffers::Buffers`])
//! - **error-resistant:** erroneous input won't abort parsing, instead
//!   notifying the caller and continuing on (see [`Callbacks`])
//! - **performance:** parsing should be reasonably fast, guaranteeing `O(n)`
//!   time complexity with no backtracking
//!
//! # Getting Started
//!
//! The typical entry point to this crate is via the [`parse()`] function. This
//! gives you an iterator over the [`GCode`]s in a string of text, ignoring any
//! errors or comments that may appear along the way.
//!
//! ```rust
//! use gcode::Mnemonic;
//!
//! let src = "G90 \n G00 X50.0 Y-10";
//!
//! let got: Vec<_> = gcode::parse(src).collect();
//!
//! assert_eq!(got.len(), 2);
//!
//! let g90 = &got[0];
//! assert_eq!(g90.mnemonic(), Mnemonic::General);
//! assert_eq!(g90.major_number(), 90);
//! assert_eq!(g90.minor_number(), 0);
//!
//! let rapid_move = &got[1];
//! assert_eq!(rapid_move.mnemonic(), Mnemonic::General);
//! assert_eq!(rapid_move.major_number(), 0);
//! assert_eq!(rapid_move.value_for('X'), Some(50.0));
//! assert_eq!(rapid_move.value_for('y'), Some(-10.0));
//! ```
//!
//! The [`full_parse_with_callbacks()`] function can be used if you want access
//! to [`Line`] information and to be notified on any parse errors.
//!
//! ```rust
//! use gcode::{Callbacks, Span};
//!
//! #[derive(Debug, Default)]
//! struct Errors {
//!     unexpected_line_number : usize,
//!     letter_without_number: usize,
//!     garbage: Vec<String>,
//! }
//!
//! impl Callbacks for Errors {
//!     fn unknown_content(&mut self, text: &str, _span: Span) {
//!         self.garbage.push(text.to_string());
//!     }
//!
//!     fn unexpected_line_number(&mut self, _line_number: f32, _span: Span) {
//!         self.unexpected_line_number += 1;
//!     }
//!
//!     fn letter_without_a_number(&mut self, _value: &str, _span: Span) {
//!         self.letter_without_number += 1;
//!     }
//! }
//!
//! let src = r"
//!     G90 N1           ; Line numbers (N) should be at the start of a line
//!     G                ; there was a G, but no number
//!     G01 X50 $$%# Y20 ; invalid characters are ignored
//! ";
//!
//! let mut errors = Errors::default();
//!
//! {
//!     let lines: Vec<_> = gcode::full_parse_with_callbacks(src, &mut errors)
//!         .collect();
//!     
//!     assert_eq!(lines.len(), 3);
//!     let total_gcodes: usize = lines.iter()
//!         .map(|line| line.gcodes().len())
//!         .sum();
//!     assert_eq!(total_gcodes, 2);
//! }
//!
//! assert_eq!(errors.unexpected_line_number, 1);
//! assert_eq!(errors.letter_without_number, 1);
//! assert_eq!(errors.garbage.len(), 1);
//! assert_eq!(errors.garbage[0], "$$%# ");
//! ```
//!
//! # Customising Memory Usage
//!
//! You'll need to manually create a [`Parser`] if you want control over buffer
//! sizes instead of relying on [`buffers::DefaultBuffers`].
//!
//! You shouldn't normally need to do this unless you are on an embedded device
//! and know your expected input will be bigger than
//! [`buffers::SmallFixedBuffers`] will allow.
//!
//! ```rust
//! use gcode::{Word, Comment, GCode, Nop, Parser, buffers::Buffers};
//! use arrayvec::ArrayVec;
//!
//! enum MyBuffers {}
//!
//! impl<'input> Buffers<'input> for MyBuffers {
//!     type Arguments = ArrayVec<[Word; 10]>;
//!     type Commands = ArrayVec<[GCode<Self::Arguments>; 2]>;
//!     type Comments = ArrayVec<[Comment<'input>; 1]>;
//! }
//!
//! let src = "G90 G01 X5.1";
//!
//! let parser: Parser<Nop, MyBuffers> = Parser::new(src, Nop);
//!
//! let lines = parser.count();
//! assert_eq!(lines, 1);
//! ```
//!
//! # Spans
//!
//! Something that distinguishes this crate from a lot of other g-code parsers
//! is that each element's original location, its [`Span`], is retained and
//! passed to the caller.
//!
//! This is important for applications such as:
//!
//! - Indicating where in the source text a parsing error or semantic error has
//!   occurred
//! - Visually highlighting the command currently being executed when stepping
//!   through a program in a simulator
//! - Reporting what point a CNC machine is up to when executing a job
//!
//! It's pretty easy to check whether something contains its [`Span`], just look
//! for a `span()` method (e.g. [`GCode::span()`]) or a `span` field (e.g.
//! [`Comment::span`]).
//!
//! # Cargo Features
//!
//! Additional functionality can be enabled by adding feature flags to your
//! `Cargo.toml` file:
//!
//! - **std:** adds `std::error::Error` impls to any errors and switches to
//!   `Vec` for the default backing buffers
//! - **serde-1:** allows serializing and deserializing most types with `serde`
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
    intra_doc_link_resolution_failure,
    missing_docs
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(test, not(feature = "std")))]
#[macro_use]
extern crate std;

#[macro_use]
mod macros;

pub mod buffers;
mod callbacks;
mod comment;
mod gcode;
mod lexer;
mod line;
mod parser;
mod span;
mod words;

pub use crate::{
    callbacks::{Callbacks, Nop},
    comment::Comment,
    gcode::{GCode, Mnemonic},
    line::Line,
    parser::{full_parse_with_callbacks, parse, Parser},
    span::Span,
    words::Word,
};
