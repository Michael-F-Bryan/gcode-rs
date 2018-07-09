//! A `no_std` gcode parsing library.

#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

mod parse;
mod types;

pub use parse::parse;
pub use types::*;
