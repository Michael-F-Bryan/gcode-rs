//! Bindings for exposing the `gcode` crate as a C library.

#![allow(unsafe_code, missing_docs)]

#[cfg(not(feature = "std"))]
compile_error!("The C bindings require the `std` feature");

use std::os::raw::{c_char, c_int, c_void};
use crate::{Parser, Span, TokenKind, Gcode, Comment, Block, Argument, Mnemonic};

/// The various possible outcomes for an operation.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub enum ParseResult {
    /// The operation was successful.
    Success = 0,
    /// An invalid argument was provided (e.g. null pointer or invalid UTF-8 
    /// strings).
    InvalidArgument = 1,
    /// The underlying Rust code panicked and wasn't able to continue. This is a
    /// bug.
    Panic = 2,
}

/// Parse a UTF-8 string as gcode.
#[no_mangle]
pub unsafe extern "C" fn parse_gcode(gcode: *const c_char, length: c_int, callbacks: Callbacks) -> ParseResult {
    if gcode.is_null() {
        return ParseResult::InvalidArgument;
    }

    let src = std::slice::from_raw_parts(gcode as *const u8, length as usize);
    let src = match std::str::from_utf8(src) {
        Ok(s) => s,
        Err(_) => return ParseResult::InvalidArgument,
    };

    let got = std::panic::catch_unwind(|| {
        for block in Parser::new_with_callbacks(src, callbacks) {
            callbacks.on_start_block(&block);

            let items = OrderedMerge {
                commands: block.commands(),
                comments: block.comments(),
            };

            for item in items {
                match item {
                    Either::Left(cmd) => callbacks.on_command(cmd),
                    Either::Right(comment) => callbacks.on_comment(comment),
                }
            }
        }
    });

    match got {
        Ok(_) => ParseResult::Success,
        Err(_) => ParseResult::Panic,
    }
}

pub type UnexpectedEOF = unsafe extern "C" fn(user_data: *mut c_void, expected: *const TokenKind, expected_len: c_int);
pub type MangledInput = unsafe extern "C" fn(user_data: *mut c_void, input: *const c_char, input_len: c_int, span: Span);
pub type UnexpectedToken = unsafe extern "C" fn(user_data: *mut c_void, found: TokenKind, span: Span, expected: *const TokenKind, expected_len: c_int);
pub type BlockStarted = unsafe extern "C" fn(user_data: *mut c_void, line_number: c_int, deleted: c_int, span: Span);
pub type ParsedGcode = unsafe extern "C" fn(user_data: *mut c_void, line_number: c_int, mnemonic: Mnemonic, major_number: c_int, minor_number: c_int, span: Span, arguments: *const Argument, argument_len: c_int);
pub type ParsedComment = unsafe extern "C" fn(user_data: *mut c_void, span: Span, body: *const c_char, body_len: c_int);

/// A set of callbacks used to notify of progress when parsing gcodes.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Callbacks {
    /// A pointer to some arbitrary data provided by the user.
    pub user_data: *mut c_void,
    /// An unexpected EOF was encountered.
    pub on_unexpected_eof: Option<UnexpectedEOF>,
    /// Skipping unparseable input.
    pub on_mangled_input: Option<MangledInput>,
    /// Encountered a token which wasn't expected.
    pub on_unexpected_token: Option<UnexpectedToken>,
    /// The parser started parsing a new block.
    pub on_start_block: Option<BlockStarted>,
    /// Parsed a g-code.
    pub on_gcode: Option<ParsedGcode>,
    /// Parsed a comment.
    pub on_comment: Option<ParsedComment>,
}

impl Callbacks {
    fn on_start_block(self, block: &Block<'_>) {
        if let Some(cb) = self.on_start_block {
            unsafe {
                let line_number = match block.line_number() {
                    Some(n) => n as c_int,
                    None => -1,
                };
                let deleted = if block.deleted() { 1 } else { 0 };
                cb(self.user_data, line_number, deleted, block.span());
            }
        }
    }

    fn on_command(self, command: &Gcode) {
        if let Some(cb) = self.on_gcode {
            unsafe {
                let line_number = match command.line_number() {
                    Some(n) => n as c_int,
                    None => -1,
                };
                let args = command.args();
                cb(self.user_data, 
                   line_number, 
                   command.mnemonic(), 
                   command.major_number() as c_int, 
                   command.minor_number().unwrap_or(0) as c_int, 
                   command.span(), 
                   args.as_ptr(), 
                   args.len() as c_int);
            }
        }
    }

    fn on_comment(self, comment: &Comment<'_>) {
        if let Some(cb) = self.on_comment {
            unsafe {
                let body = comment.body();
                cb(self.user_data, comment.span(), body.as_ptr() as *const c_char, body.len() as c_int);
            }
        }
    }
}

impl crate::Callbacks for Callbacks {
    fn unexpected_token(
    &mut self,
    found: TokenKind,
    span: Span,
    expected: &[TokenKind]
    ) {
        if let Some(cb) = self.on_unexpected_token {
            unsafe {
                cb(self.user_data, found, span, expected.as_ptr(), expected.len() as c_int);
            }
        }
    }

    fn unexpected_eof(&mut self, expected: &[TokenKind]) {
        if let Some(cb) = self.on_unexpected_eof {
            unsafe {
                cb(self.user_data, expected.as_ptr(), expected.len() as c_int);
            }
        }
    }

    fn mangled_input(&mut self, input: &str, span: Span) {
        if let Some(cb) = self.on_mangled_input {
            unsafe {
                cb(self.user_data, input.as_ptr() as *const c_char, input.len() as c_int, span);
            }
        }
    }
}

enum Either<A, B> {
    Left(A),
    Right(B),
}

/// An iterator that takes a list of gcodes and comments, and merges them based
/// on where they appear in the input text.
struct OrderedMerge<'a> {
    commands: &'a [Gcode],
    comments: &'a [Comment<'a>],
}

impl<'a> OrderedMerge<'a> {
    fn pop_earliest(&mut self) -> Either<&'a Gcode, &'a Comment<'a>> {
        let cmd_span = self.commands[0].span();
        let comment_span = self.comments[0].span();

        if cmd_span.start <= comment_span.start {
            self.pop_command()
        } else {
            self.pop_comment()
        }
    }

    fn pop_command(&mut self) -> Either<&'a Gcode, &'a Comment<'a>> {
        let cmd = &self.commands[0];
        self.commands = &self.commands[1..];
        Either::Left(cmd)
    }

    fn pop_comment(&mut self) -> Either<&'a Gcode, &'a Comment<'a>> {
        let comment = &self.comments[0];
        self.comments = &self.comments[1..];
        Either::Right(comment)
    }
}

impl<'a> Iterator for OrderedMerge<'a> {
    type Item = Either<&'a Gcode, &'a Comment<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.commands.is_empty(), self.comments.is_empty()) {
            (false, false) => Some(self.pop_earliest()),
            (false, true) => Some(self.pop_command()),
            (true, false) => Some(self.pop_comment()),
            (true, true) => None,
        }
    }
}