//! A FFI interface to the `gcode` crate.
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
use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_float, c_int};
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
    src: *const c_char,
    src_len: c_int,
) -> *mut Parser {
    if src.is_null() {
        return ptr::null_mut();
    }

    // first, turn the input into a proper UTF-8 string
    let src = slice::from_raw_parts(src as *const u8, src_len as usize);
    let src = match str::from_utf8(src) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(Parser(gcode::parse(src))))
}

/// Destroy the `Parser` once you no longer need it.
#[no_mangle]
pub unsafe extern "C" fn parser_destroy(parser: *mut Parser) {
    if parser.is_null() {
        return;
    }

    let boxed = Box::from_raw(parser);
    drop(boxed);
}

/// Allocate a new, empty `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_new() -> *mut Gcode {
    Box::into_raw(Box::new(Gcode(None)))
}

/// Free the `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_destroy(gcode: *mut Gcode) {
    if gcode.is_null() {
        return;
    }

    let boxed = Box::from_raw(gcode);
    drop(boxed);
}

/// Get the next `Gcode`, returning `false` when there are no more `Gcode`s in
/// the input.
///
/// # Note
///
/// To avoid unnecessary allocations, you can either pass in a newly created
/// `Gcode` or reuse an existing one.
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
pub unsafe extern "C" fn gcode_major_number(gcode: *const Gcode) -> c_int {
    (&*gcode).major_number() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn gcode_minor_number(gcode: *const Gcode) -> c_int {
    (&*gcode).minor_number().unwrap_or(0) as c_int
}

/// The number of arguments in this `Gcode`.
#[no_mangle]
pub unsafe extern "C" fn gcode_arg_count(gcode: *const Gcode) -> c_int {
    (&*gcode).args().len() as c_int
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
    value: *mut c_float,
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
    line_number: *mut c_int,
) -> bool {
    match (&*gcode).line_number() {
        Some(n) => {
            *line_number = n as c_int;
            true
        }
        None => false,
    }
}
