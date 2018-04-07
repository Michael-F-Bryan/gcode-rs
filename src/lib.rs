//! A `no_std` gcode parsing library.

#![no_std]

#[macro_use]
extern crate nom;

#[cfg(not(test))]
extern crate core as std;
#[cfg(test)]
#[macro_use]
extern crate std;

mod parse;
pub mod types;
