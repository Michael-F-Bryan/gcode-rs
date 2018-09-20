//! A FFI interface to the `gcode` library.
//!
//! # Error Handling
//!
//! Most functions will return a boolean `success` value to indicate whether
//! they were successful or not.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![allow(missing_docs, unsafe_code)]

extern crate gcode;

use gcode::{Gcode as _Gcode, Parser as _Parser};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::str;

pub use gcode::{Mnemonic, Span, Word};

#[derive(Debug)]
pub struct Parser(_Parser<'static>);

impl Deref for Parser {
    type Target = _Parser<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Parser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An opaque `Gcode`.
#[derive(Debug)]
pub struct Gcode(Option<_Gcode>);

impl Deref for Gcode {
    type Target = _Gcode;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("The `Gcode` should have already been created")
    }
}

impl DerefMut for Gcode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("The `Gcode` should have already been created")
    }
}

/// Create a new parser.
///
/// # Safety
///
/// In order to maintain memory safety, the `Parser` must not outlive the
/// source string.
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

    Box::into_raw(Box::new(Parser(gcode::parse(src))))
}

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
///
/// # Note
///
/// You can either pass in a newly created `Gcode` or reuse an existing one.
#[no_mangle]
pub unsafe extern "C" fn parser_next(
    parser: *mut Parser,
    gcode: *mut Gcode,
) -> bool {
    let parser = &mut *parser;
    let gcode = &mut *gcode;

    match parser.next() {
        Some(got) => {
            gcode.0 = Some(got);
            true
        }
        None => false,
    }
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

#[no_mangle]
pub unsafe extern "C" fn gcode_minor_number(gcode: *const Gcode) -> u32 {
    (&*gcode).minor_number().unwrap_or(0)
}

/// The number of arguments in this `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_arg_count(gcode: *const Gcode) -> i32 {
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
            *value = n;
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
