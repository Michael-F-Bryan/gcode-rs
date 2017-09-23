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

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

extern crate arrayvec;

mod helpers;

pub mod errors {
    use super::*;

    /// An alias for the `Result` type.
    pub type Result<T> = ::core::result::Result<T, Error>;

    /// The error type.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Error {}
}
