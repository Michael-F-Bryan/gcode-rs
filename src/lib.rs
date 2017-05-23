#![no_std]

// Required because we're using `core::num::Float` and `f32::powi()` in "parser.rs"
#![feature(core_float)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod parser;
mod commands;

pub use commands::{Argument, G};
pub use parser::{Parser, Instructions};
pub use errors::*;

mod errors {
    pub type Result<T> = ::core::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        Expected(char),
        UnexpectedEOF,
    }
}
