#![cfg_attr(not(feature = "std"), no_std)]

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
mod types;

pub use parser::*;
pub use types::*;

/// Convenience function for parsing a string of text into `Gcode`s, ignoring
/// any errors which may occur.
pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src).flat_map(|block| block.into_commands())
}
