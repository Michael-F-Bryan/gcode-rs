//! C bindings to the `gcode` crate.
#![deny(
    bare_trait_objects,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

use gcode::{Comment, GCode, Mnemonic, Span, Word};
use std::{
    os::raw::{c_char, c_int, c_void},
    slice,
};

/// Parse the provided string to the end, triggering callbacks when gcodes and
/// comments are encountered.
#[no_mangle]
pub unsafe extern "C" fn parse_gcode(
    src: *const c_char,
    len: c_int,
    vtable: VTable,
) -> ParseResult {
    if src.is_null() || len < 0 {
        return ParseResult::InvalidArgument;
    }

    let buffer = slice::from_raw_parts(src as *const u8, len as usize);

    let src = match std::str::from_utf8(buffer) {
        Ok(b) => b,
        Err(_) => return ParseResult::InvalidUTF8,
    };

    for line in gcode::parse_with_callbacks(src, Callbacks(vtable)) {
        vtable.on_line_start(line.line_number().map(|w| w.value), line.span());
        vtable.handle_gcodes(line.gcodes());
        vtable.handle_comments(line.comments());
    }

    ParseResult::Success
}

/// The outcome of parsing.
///
/// Note that the parser itself is quite tolerant of garbled input text (e.g.
/// unrecognised characters or invalid syntax), so in general the only time when
/// it'll fail outright is due to incorrect arguments.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum ParseResult {
    /// The operation was successful.
    Success = 0,
    /// The input text is invalid UTF-8.
    InvalidUTF8 = 1,
    /// An invalid argument was provided (e.g. the string is a null pointer or
    /// the length isn't non-negative).
    InvalidArgument = 2,
}

#[derive(Debug, Copy, Clone)]
struct Callbacks(VTable);

impl gcode::Callbacks for Callbacks {
    fn unknown_content(&mut self, text: &str, span: Span) {
        if let Some(cb) = self.0.on_unknown_content {
            unsafe {
                cb(
                    self.0.user_data,
                    text.as_ptr() as *const c_char,
                    text.len() as c_int,
                    span,
                );
            }
        }
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        if let Some(cb) = self.0.on_unexpected_line_number {
            unsafe {
                cb(self.0.user_data, line_number, span);
            }
        }
    }

    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: Span,
    ) {
        if let Some(cb) = self.0.on_argument_without_a_command {
            unsafe {
                cb(self.0.user_data, letter, value, span);
            }
        }
    }

    fn number_without_a_letter(&mut self, value: &str, span: Span) {
        if let Some(cb) = self.0.on_number_without_a_letter {
            unsafe {
                cb(
                    self.0.user_data,
                    value.as_ptr() as *const c_char,
                    value.len() as c_int,
                    span,
                );
            }
        }
    }

    fn letter_without_a_number(&mut self, value: &str, span: Span) {
        if let Some(cb) = self.0.on_letter_without_a_number {
            unsafe {
                cb(
                    self.0.user_data,
                    value.as_ptr() as *const c_char,
                    value.len() as c_int,
                    span,
                );
            }
        }
    }
}

/// A bunch of function pointers fired during the parsing process to notify the
/// caller when "something" happens.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct VTable {
    /// Arbitrary data which will be passed to each callback.
    pub user_data: *mut c_void,
    /// The start of a line.
    pub on_line_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            line_number: c_int,
            span: Span,
        ),
    >,
    /// Parsed a comment.
    pub on_comment: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            body: *const c_char,
            len: c_int,
            span: Span,
        ),
    >,
    /// Parsed a gcode.
    pub on_gcode: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            mnemonic: Mnemonic,
            major_number: c_int,
            minor_number: c_int,
            args: *const Word,
            arg_len: c_int,
            span: Span,
        ),
    >,
    /// Encountered an unknown string of text.
    pub on_unknown_content: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            text: *const c_char,
            text_len: c_int,
            span: Span,
        ),
    >,
    /// Encountered a line number where it wasn't expected.
    pub on_unexpected_line_number: Option<
        unsafe extern "C" fn(user_data: *mut c_void, value: f32, span: Span),
    >,
    /// Encountered an argument without any previous argument.
    pub on_argument_without_a_command: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            letter: char,
            value: f32,
            span: Span,
        ),
    >,
    /// A number was encountered that wasn't preceeded by a letter.
    pub on_number_without_a_letter: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            text: *const c_char,
            text_len: c_int,
            span: Span,
        ),
    >,
    /// A letter was encountered that wasn't followed by a number.
    pub on_letter_without_a_number: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            text: *const c_char,
            text_len: c_int,
            span: Span,
        ),
    >,
}

impl VTable {
    fn on_line_start(&self, line_number: Option<f32>, span: Span) {
        let line_number = line_number.map(|f| f.floor() as c_int).unwrap_or(-1);

        if let Some(cb) = self.on_line_start {
            unsafe {
                cb(self.user_data, line_number, span);
            }
        }
    }

    fn handle_comments(&self, comments: &[Comment<'_>]) {
        if let Some(cb) = self.on_comment {
            for comment in comments {
                unsafe {
                    cb(
                        self.user_data,
                        comment.value.as_ptr() as *const c_char,
                        comment.value.len() as c_int,
                        comment.span,
                    );
                }
            }
        }
    }

    fn handle_gcodes(&self, gcodes: &[GCode]) {
        if let Some(cb) = self.on_gcode {
            for gcode in gcodes {
                let args = gcode.arguments();
                unsafe {
                    cb(
                        self.user_data,
                        gcode.mnemonic(),
                        gcode.major_number() as c_int,
                        gcode.minor_number() as c_int,
                        args.as_ptr(),
                        args.len() as c_int,
                        gcode.span(),
                    );
                }
            }
        }
    }
}
