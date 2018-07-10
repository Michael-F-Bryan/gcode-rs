//! A `no_std` gcode parsing library.

#![no_std]

extern crate arrayvec;

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod lexer;
mod parse;
mod types;

pub use parse::parse;
pub use types::*;
