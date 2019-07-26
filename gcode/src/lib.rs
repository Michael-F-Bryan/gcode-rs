#![cfg_attr(not(feature = "std"), no_std)]

mod comment;
mod lexer;
mod span;
mod words;

pub use crate::comment::Comment;
pub use crate::span::Span;
pub use crate::words::Word;

#[cfg(feature = "std")]
pub use crate::comment::OwnedComment;
