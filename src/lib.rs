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

pub mod parse;
mod validate;

pub use validate::validate;
