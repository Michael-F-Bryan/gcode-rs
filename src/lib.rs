#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

mod parser;
mod lexer;
mod helpers;

pub use lexer::Tokenizer;
pub use parser::Parser;
pub use errors::*;

mod errors {
    pub type Result<T> = ::core::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        UnknownToken(char),
        /// A number was provided which doesn't contain a decimal point.
        InvalidNumber,
        UnexpectedEOF,
    }
}
