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
    variant_size_differences,
    intra_doc_link_resolution_failure
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(test, not(feature = "std")))]
#[macro_use]
extern crate std;

#[macro_use]
mod macros;

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
