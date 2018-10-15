#![feature(try_from)]
#![no_std]

extern crate gcode;
extern crate libm;

#[cfg(test)]
#[macro_use]
pub extern crate std;
#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod operations;
pub mod state;

pub use operations::Operation;
pub use state::State;
