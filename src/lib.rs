//! A crate for parsing gcodes without `std`.

#![no_std]
#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod parser;
mod lexer;
mod helpers;

pub use lexer::{Token, Tokenizer};
pub use parser::Parser;
pub use errors::*;

mod errors {
    /// An alias for the `Result` type.
    pub type Result<T> = ::core::result::Result<T, Error>;

    /// Any error which may be returned by this crate.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        /// Encountered an unknown token.
        UnknownToken(char),
        /// Reached the end of input, unexpectedly.
        UnexpectedEOF,
    }
}
