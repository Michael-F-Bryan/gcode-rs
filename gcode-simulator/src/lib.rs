extern crate gcode;
extern crate id_arena;
extern crate libm;
extern crate sum_type;
extern crate uom;

#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod operations;
pub mod sim;
pub mod state;

pub use crate::{operations::Operation, state::State};

/// A stand-in for the currently unstable `std::convert::TryFrom` trait.
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(other: T) -> Result<Self, Self::Error>;
}
