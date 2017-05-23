#![no_std]
#![feature(core_float)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod parser;
mod commands;

pub use commands::Argument;
pub use parser::Parser;
pub use errors::*;

mod errors {
    pub type Result<T> = ::core::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        Expected(char),
        UnexpectedEOF,
    }
}
