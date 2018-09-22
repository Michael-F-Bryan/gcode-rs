#![cfg_attr(not(feature = "std"), no_std)]

extern crate arrayvec;
#[cfg(not(feature = "std"))]
extern crate libm;

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

pub mod types;
