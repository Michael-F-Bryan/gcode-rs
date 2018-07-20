//! A FFI interface to the `gcode` library.
//!
//! # Examples
//!
//! ```rust
//! use gcode::ffi::{self, SIZE_OF_PARSER, SIZE_OF_GCODE, Parser, Gcode};
//!
//! // allocate some space on the stack for our parser. Normally you'd just use
//! // malloc(), but because we don't have an allocator, we create an
//! // appropriately sized byte buffer and use pointer casts to "pretend" 
//! // it's the right thing.
//! let mut parser = [0_u8; SIZE_OF_PARSER];
//! let parser = parser.as_mut_ptr() as *mut Parser;
//!
//! let src = "G01 X-52.4";
//!
//! unsafe {
//!     let ret = ffi::parser_new(parser, src.as_ptr(), src.len() as i32);
//!     assert_eq!(ret, 0, "Creation failed");
//!
//!     let mut code = [0_u8; SIZE_OF_GCODE];
//!     let code = code.as_mut_ptr() as *mut Gcode;
//!
//!     ffi::parser_destroy(parser);
//! }
//! ```

#![allow(missing_docs, unsafe_code)]

use core::mem;
use core::str;
use core::ptr;
use core::slice;
use parse::Parser as MyParser;
use types::Gcode as MyGcode;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Parser {
    __cant_create: (),
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Gcode {
    __cant_create: (),
}

pub const SIZE_OF_PARSER: usize = mem::size_of::<MyParser>();
pub const SIZE_OF_GCODE: usize = mem::size_of::<MyGcode>();

/// Create a new parser.
///
/// # Safety
///
/// In order to maintain memory safety, there are two invariants which **must**
/// be upheld:
///
/// 1. The `Parser` must not outlive the source string
/// 2. You must allocate `SIZE_OF_PARSER` bytes (e.g. as an array on the stack)
///    for the `Parser` to be placed in
#[no_mangle]
pub unsafe extern "C" fn parser_new(parser: *mut Parser, src: *const u8, src_len: i32) -> i32 {
    if src.is_null() || parser.is_null() {
        return -1;
    }

    // first, turn the input into a proper UTF-8 string
    let src = slice::from_raw_parts(src, src_len as usize);
    let src = match str::from_utf8(src) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // create our parser, Parser<'input>
    let local_parser = MyParser::new(src);
    
    // Copy it to the destination
    ptr::copy_nonoverlapping(&local_parser, parser as *mut MyParser, 1);
    
    // it is now the caller's responsibility to clean up `parser`, forget our
    // local copy
    mem::forget(local_parser);

    0
}

/// Destroy a `Parser` once it is no longer needed.
#[no_mangle]
pub unsafe extern "C" fn parser_destroy(parser: *mut Parser) {
    if parser.is_null() {
        return;
    }

    ptr::drop_in_place(parser as *mut MyParser);
}
