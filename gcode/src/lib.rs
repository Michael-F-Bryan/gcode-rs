#![cfg_attr(not(feature = "std"), no_std)]

extern crate arrayvec;
extern crate libm;

#[cfg(all(not(feature = "std"), test))]
#[macro_use]
extern crate std;

mod lexer;
pub mod types;
