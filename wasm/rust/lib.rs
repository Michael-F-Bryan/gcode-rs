//! Internals for the `@michael-f-bryan/gcode` package. Not intended for public
//! use.

mod callbacks;
mod parser;
mod simple_wrappers;

pub use callbacks::JavaScriptCallbacks;
pub use parser::Parser;
pub use simple_wrappers::{Comment, GCode, Line, Span, Word};

use gcode::Mnemonic;

pub(crate) fn mnemonic_letter(m: Mnemonic) -> char {
    match m {
        Mnemonic::General => 'G',
        Mnemonic::Miscellaneous => 'M',
        Mnemonic::ProgramNumber => 'O',
        Mnemonic::ToolChange => 'T',
    }
}