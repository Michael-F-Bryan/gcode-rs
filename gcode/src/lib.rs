//! A gcode parsing library designed for `no_std` environments.
//!
//! # Feature Flags
//!
//! To help reduce compilation times and overall code size, this crate puts
//! extra functionality behind several `cargo` features.
//!
//! - `std`: Enables various features/optimisations which require allocation
//!   or implementing traits from `std` (e.g. `std::error::Error`).
//! - `transforms`: Exposes the transformations API for manipulating `Gcode`s
//!   before executing or writing to a file.
//! - `large-buffers` (on-by-default): Increases the number of commands,
//!   comments and arguments which can be added to a block (see
//!   [`Block::MAX_COMMENT_COUNT`], [`Block::MAX_COMMAND_COUNT`], and
//!   [`Gcode::MAX_ARGUMENT_COUNT`] for more).

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    rust_2018_idioms,
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

#[cfg(feature = "builder")]
pub mod builder;
mod lexer;
mod parser;
#[cfg(feature = "transforms")]
pub mod transforms;
mod types;

#[cfg(feature = "transforms")]
pub use crate::transforms::GcodeTransforms;
pub use crate::{parser::*, types::*};

/// Convenience function for parsing a string of text into `Gcode`s, ignoring
/// any errors which may occur.
pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src).flat_map(|block| block.into_commands())
}
