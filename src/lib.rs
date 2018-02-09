//! A library for parsing gcodes in an embedded program.
//!
//! # Examples
//!
//! ```rust
//! # use gcode::Parser;
//! let mut parser = Parser::new();
//! let src = b"G90 X10.0 Y73.0 Z0.5\nN20 G91 Z1.0";
//!
//! for command in parser.parse(src) {
//!   // do something with the command
//! }
//! ```

#![no_std]
// #![deny(missing_docs,
//         missing_debug_implementations,
//         missing_copy_implementations,
//         trivial_casts,
//         trivial_numeric_casts,
//         unsafe_code,
//         unused_import_braces,
//         unused_qualifications,
//         unstable_features)]
#![allow(unused_variables, dead_code, unused_extern_crates, unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

extern crate arrayvec;

#[macro_use]
mod macros;
mod helpers;
mod parser;
mod command;

pub use parser::Parser;
pub use command::{Arguments, Command, Kind};

pub mod errors {
    use super::*;

    /// An alias for the `Result` type.
    pub type Result<T> = ::core::result::Result<T, Error>;

    /// The error type.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Error {}
}
