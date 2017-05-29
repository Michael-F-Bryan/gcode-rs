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
//! Stage | Transform                      | Transformer     | Description
//! ------+--------------------------------+--------+----------------------------------
//! 1     | char -> [`Token`]              | [`Tokenizer`]   | Lexical analysis (tokenizing)
//! 2     | Token -> [`low_level::Line`]   | [`BasicParser`] | Initial parsing
//! 3     | Line -> [`high_level::Line`]   | [`Parser`]      | Type-checking
//!
//!
//! [`Iterator`]: https://doc.rust-lang.org/nightly/core/iter/trait.Iterator.html
//! [`Tokenizer`]: lexer/struct.Tokenizer.html
//! [`BasicParser`]: parser/struct.BasicParser.html
//! [`Parser`]: high_level/struct.Parser.html
//! [`Token`]: lexer/struct.Token.html
//! [`low_level::Line`]: low_level/enum.Line.html
//! [`high_level::Line`]: high_level/enum.Line.html

#![no_std]
#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate arrayvec;

pub mod low_level;
pub mod lexer;
mod helpers;
pub mod high_level;

pub use lexer::Span;
pub use low_level::BasicParser;
pub use errors::*;

mod errors {
    use super::*;

    /// An alias for the `Result` type.
    pub type Result<T> = ::core::result::Result<T, Error>;

    /// The error type.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        /// Encountered an unknown token at a particular location.
        UnknownToken(char, Span),
        /// Reached the end of input, unexpectedly.
        UnexpectedEOF,

        /// A syntax error and its location.
        SyntaxError(&'static str, Span),
    }
}
