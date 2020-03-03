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

mod comment;
mod gcode;
mod lexer;
mod lines;
mod span;
mod words;
mod buffers;

pub use crate::{
    comment::Comment,
    gcode::{GCode, Mnemonic},
    lines::{
        parse, parse_with_callbacks, Callbacks, Line, MAX_COMMAND_LEN,
        MAX_COMMENT_LEN,
    },
    span::Span,
    words::Word,
    buffers::{Buffers, Buffer, VecBuffers, SmallFixedBuffers, DefaultBuffers},
};

#[cfg(feature = "std")]
pub use crate::comment::OwnedComment;
