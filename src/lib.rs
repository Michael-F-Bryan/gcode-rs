//! A crate for parsing gcodes without relying on `std`.
//!
//! The crate uses iterators extensively to implement a zero-allocation lexer
//! and parser.
//!
//!
//! # Examples
//!
//! You can manually exercise the entire pipeline as follows:
//!
//! ```rust
//! use gcode::{Tokenizer, Parser};
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
//! let parser = Parser::new(tokens);
//!
//! // Skip all parsing errors and then apply type checking, skipping errors again
//! let lines = parser.filter_map(|l| l.ok());
//!
//! for line in lines {
//!     println!("{:?}", line);
//! }
//! ```
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
        missing_debug_implementations,
        missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unsafe_code,
        unused_import_braces,
        unused_qualifications,
        unstable_features)]
#![allow(deprecated)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

extern crate arrayvec;

#[deprecated(since="0.2.0", note="Please use the `parser` module instead")]
pub mod low_level;
pub mod lexer;
mod helpers;
pub mod parser;

pub use parser::Parser;
pub use lexer::{Tokenizer, Span};
pub use low_level::BasicParser;
pub use errors::*;


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
