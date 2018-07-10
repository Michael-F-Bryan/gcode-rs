//! A `no_std` gcode parsing library.
//!
//! # Examples
//!
//! ```rust
//! use gcode::Mnemonic;
//!
//! let src = "O1000
//!     T1 M6
//!     G90 
//!     G01 X-75 Y-75 S500 M3 
//!     G43 Z100 H1
//!     G01 Z5
//!     N20 G01 Z-20 F100";
//!
//! let mut lines = gcode::parse(src);
//!
//! let program_number = lines.next().unwrap();
//! assert_eq!(program_number.major_number(), 1000);
//!
//! let tool_change = lines.next().unwrap();
//! assert_eq!(tool_change.mnemonic(), Mnemonic::ToolChange);
//! assert_eq!(tool_change.major_number(), 1);
//!
//! // skip the M6 and G90
//! let _ = lines.next();
//! let _ = lines.next();
//!
//! let g01 = lines.next().unwrap();
//! assert_eq!(g01.major_number(), 1);
//! assert_eq!(g01.args().len(), 3);
//! assert_eq!(g01.value_for('X'), Some(-75.0));
//!
//! let rest: Vec<_> = lines.collect();
//! assert_eq!(rest.len(), 4);
//! assert_eq!(rest[3].line_number(), Some(20));
//! ```


#![no_std]
#![deny(missing_docs,
        missing_debug_implementations, 
        missing_copy_implementations,
        trivial_casts, 
        trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, 
        unused_qualifications)]

extern crate arrayvec;

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod lexer;
mod parse;
mod types;

pub use parse::parse;
pub use types::*;
