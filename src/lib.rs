//! A crate for parsing gcodes without relying on `std`.
//!
//! The crate uses a pipeline pattern and iterators to implement a
//! zero-allocation lexer and parser.
//!
//!
//! # Pipeline Stages
//!
//! The data goes through a series of transformations before it arrives at its
//! final strongly-typed representation. Each transform uses the [`Iterator`]
//! trait to lazily evaluate commands, so if you aren't reading from local
//! memory (i.e. an attached SD card), if you're reading data from a serial
//! port or other network device you'll probably want to wrap it in some sort
//! of buffered reader.
//!
//!
//! Stage | Transform                      | Transformer       | Description
//! ------+--------------------------------+--------+------------------------------------
//! 1     | char -> [`Token`]              | [`Tokenizer`]     | Lexical analysis (tokenizing)
//! 2     | Token -> [`low_level::Line`]   | [`BasicParser`]   | Initial parsing
//! 3     | Line -> [`high_level::Line`]   | [`type_check()`]  | Type-checking and strong typing
//!
//!
//! # Examples
//!
//! You can manually exercise the entire pipeline as follows:
//!
//! ```rust
//! use gcode::{Tokenizer, BasicParser};
//!
//! let src = "G00 X10.0 Y20.0; G00 Z-10.0; G01 X55.2 Y-32.0 F500;";
//!
//! // Construct a tokenizer and pass it our source code.
//! let lexer = Tokenizer::new(src.chars());
//!
//! // Ignore any errors we encounter to get a stream of `Token`s.
//! let tokens = lexer.filter_map(|t| t.ok());
//!
//! // Construct a parser which takes in the tokens.
//! let parser = BasicParser::new(tokens);
//!
//! // Skip all parsing errors and then apply type checking, skipping errors again
//! let lines = parser.filter_map(|l| l.ok());
//!
//! for line in lines {
//!     println!("{:?}", line);
//! }
//! ```
//!
//! Alternatively, using the `nightly` feature flag, you can just use the
//! `parse()` function.
//!
//! ```rust,ignore
//! use gcode::parse;
//!
//! let src = "G00 X10.0 Y20.0; G00 Z-10.0; G01 X55.2 Y-32.0 F500;";
//!
//! for line in parse(src.chars()) {
//!     println!("{:?}", line);
//! }
//! ```
//!
//! # Feature Flags
//!
//! At the moment only one feature flag is available.
//!
//! - `nightly` enables the `parse` function. This is a convenience function
//!   which will convert a stream of `char`s into a stream of
//!   [`high_level::Line`]s. It relies on `conservative_impl_trait`.
//!
//!
//! [`Iterator`]: https://doc.rust-lang.org/nightly/core/iter/trait.Iterator.html
//! [`Tokenizer`]: lexer/struct.Tokenizer.html
//! [`BasicParser`]: parser/struct.BasicParser.html
//! [`type_check()`]: high_level/fn.type_check.html
//! [`Token`]: lexer/struct.Token.html
//! [`low_level::Line`]: low_level/enum.Line.html
//! [`high_level::Line`]: high_level/enum.Line.html

#![no_std]
#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unused_import_braces, unused_qualifications)]

// Allow using unstable features with the "nightly" feature flag
#![cfg_attr(not(feature = "nightly"), deny(unstable_features))]

// Enable nightly-only features
#![cfg_attr(feature = "nightly", feature(conservative_impl_trait))]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

extern crate arrayvec;

pub mod low_level;
pub mod lexer;
mod helpers;
pub mod parser;

pub use lexer::{Tokenizer, Span};
pub use low_level::BasicParser;
pub use errors::*;

#[cfg(feature = "nightly")]
pub use helpers::lines::parse;

mod errors {
    use super::*;

    /// An alias for the `Result` type.
    pub type Result<T> = ::core::result::Result<T, Error>;

    /// The error type.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Error {
        /// Encountered an unknown token at a particular location.
        UnknownToken(char, Span),
        /// Reached the end of input, unexpectedly.
        UnexpectedEOF,

        /// A syntax error and its location.
        SyntaxError(&'static str, Span),

        /// During type-checking invalid command conditions were encountered.
        InvalidCommand(&'static str),
    }
}
