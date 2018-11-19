//! G-Code parsing and manipulation.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

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
