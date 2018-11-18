#![cfg_attr(not(feature = "std"), no_std)]
#![warn(rust_2018_idioms)]
#![deny(missing_copy_implementations, missing_debug_implementations)]

extern crate arrayvec;
#[macro_use]
extern crate cfg_if;
#[cfg(feature = "std")]
extern crate core;
extern crate libm;

#[cfg(all(not(feature = "std"), test))]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod lexer;
mod parser;
pub mod transforms;
mod types;

pub use crate::parser::*;
pub use crate::transforms::GcodeTransforms;
pub use crate::types::*;

/// Convenience function for parsing a string of text into `Gcode`s, ignoring
/// any errors which may occur.
pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src).flat_map(|block| block.into_commands())
}
