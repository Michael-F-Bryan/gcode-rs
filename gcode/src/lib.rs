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

pub use crate::comment::Comment;
pub use crate::gcode::{GCode, Mnemonic};
pub use crate::lines::{
    parse, parse_with_callbacks, Callbacks, Line, MAX_COMMAND_LEN, MAX_COMMENT_LEN,
};
pub use crate::span::Span;
pub use crate::words::Word;

#[cfg(feature = "std")]
pub use crate::comment::OwnedComment;
