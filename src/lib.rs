#![no_std]
#![feature(core_float)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod parser;
mod argument;

pub use argument::Argument;
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
