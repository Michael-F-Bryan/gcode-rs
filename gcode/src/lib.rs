#![deny(
    bare_trait_objects,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unsafe_code,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(test, not(feature = "std")))]
#[macro_use]
extern crate std;

pub mod buffers;
mod comment;
mod gcode;
mod lexer;
mod lines;
mod span;
mod words;

pub use crate::{
    comment::Comment,
    gcode::{GCode, Mnemonic},
    lines::{parse, parse_with_callbacks, Callbacks, Line},
    span::Span,
    words::Word,
};

#[cfg(feature = "std")]
pub use crate::comment::OwnedComment;
