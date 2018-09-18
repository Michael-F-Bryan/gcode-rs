//! A FFI interface to the `gcode` library.
//!
//! # Error Handling
//!
//! Most functions will return a boolean `success` value to indicate whether
//! they were successful or not.
//!
//! # Examples
//!
//! ```rust
//! use gcode::ffi;
//! use gcode::{Parser, Gcode, Mnemonic};
//! use std::mem;
//!
//! let src = "O1000\nG01 X-52.4 G4 P50.0";
//!
//! unsafe {
//!     let parser = ffi::parser_new(src.as_ptr(), src.len() as i32);
//!     assert!(!parser.is_null(), "Creation failed");
//!
//!     let mut code = ffi::gcode_new();
//!     let mut num_gcodes = 0;
//!     let mut cumulative_x = 0.0;
//!     let mut cumulative_y = 0.0;
//!
//!     while ffi::parser_next(parser, code) {
//!         let mut x = 0.0;
//!         if ffi::gcode_mnemonic(code) == Mnemonic::General {
//!             num_gcodes += 1;
//!         }
//!
//!         if ffi::gcode_arg_value(code, 'X', &mut x) {
//!             cumulative_x += x;
//!         }
//!
//!         let mut y = 0.0;
//!         if ffi::gcode_arg_value(code, 'Y', &mut y) {
//!             cumulative_y += y;
//!         }
//!     }
//!
//!     assert_eq!(num_gcodes, 2);
//!     assert_eq!(cumulative_x, -52.4);
//!     assert_eq!(cumulative_y, 0.0);
//!
//!     ffi::parser_destroy(parser);
//! }
//! ```

#![allow(missing_docs, unsafe_code)]

use parse::Parser as MyParser;
use std::prelude::v1::*;
use std::ptr;
use std::slice;
use std::str;
use types::{Gcode, Mnemonic, Span, Word};

#[derive(Debug)]
pub struct Parser(MyParser<'static>);

/// Create a new parser.
///
/// # Safety
///
/// In order to maintain memory safety, the `Parser` must not outlive the source
/// string.
#[no_mangle]
pub unsafe extern "C" fn parser_new(
    src: *const u8,
    src_len: i32,
) -> *mut Parser {
    if src.is_null() {
        return ptr::null_mut();
    }

    // first, turn the input into a proper UTF-8 string
    let src = slice::from_raw_parts(src, src_len as usize);
    let src = match str::from_utf8(src) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(Parser(MyParser::new(src))))
}

/// Destroy a `Parser` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn parser_destroy(parser: *mut Parser) {
    if parser.is_null() {
        return;
    }

    let boxed = Box::from_raw(parser);
    drop(boxed);
}

/// Get the next `Gcode`, returning `false` when there are no more `Gcode`s in
/// the input.
#[no_mangle]
pub unsafe extern "C" fn parser_next(
    parser: *mut Parser,
    gcode: *mut Gcode,
) -> bool {
    let parser = &mut (*parser).0;

    match parser.next() {
        Some(got) => {
            ptr::write(gcode, got);
            true
        }
        None => false,
    }
}

/// Create a new empty `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_new() -> *mut Gcode {
    Box::into_raw(Box::new(Gcode::default()))
}

#[no_mangle]
pub unsafe extern "C" fn gcode_destroy(code: *mut Gcode) {
    if code.is_null() {
        return;
    }

    let boxed = Box::from_raw(code);
    drop(boxed);
}

/// The overall category this `Gcode` belongs to.
#[no_mangle]
pub unsafe extern "C" fn gcode_mnemonic(gcode: *const Gcode) -> Mnemonic {
    (&*gcode).mnemonic()
}

#[no_mangle]
pub unsafe extern "C" fn gcode_major_number(gcode: *const Gcode) -> u32 {
    (&*gcode).major_number()
}

/// The number of arguments in this `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_num_args(gcode: *const Gcode) -> i32 {
    (&*gcode).args().len() as i32
}

/// Get a pointer to this `Gcode`'s arguments.
#[no_mangle]
pub unsafe extern "C" fn gcode_args(gcode: *const Gcode) -> *const Word {
    (&*gcode).args().as_ptr()
}

/// Get the value for the argument with a particular letter.
#[no_mangle]
pub unsafe extern "C" fn gcode_arg_value(
    gcode: *const Gcode,
    letter: char,
    value: *mut f32,
) -> bool {
    match (&*gcode).value_for(letter) {
        Some(n) => {
            *value = f64::from(n) as f32;
            true
        }
        None => false,
    }
}

/// The `Gcode`'s location in its source code.
#[no_mangle]
pub unsafe extern "C" fn gcode_span(gcode: *const Gcode) -> Span {
    (&*gcode).span()
}

/// Get a `Gcode`'s line number (the `N20` argument), if it was assigned.
#[no_mangle]
pub unsafe extern "C" fn gcode_line_number(
    gcode: *const Gcode,
    line_number: *mut u32,
) -> bool {
    match (&*gcode).line_number() {
        Some(n) => {
            *line_number = n;
            true
        }
        None => false,
    }
}
