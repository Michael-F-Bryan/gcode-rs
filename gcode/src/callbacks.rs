use crate::{Comment, Mnemonic, Span, Word};

#[allow(unused_imports)] // rustdoc links
use crate::{buffers::Buffers, GCode};

/// Callbacks used during the parsing process to indicate possible errors.
pub trait Callbacks {
    /// The parser encountered some text it wasn't able to make sense of.
    fn unknown_content(&mut self, _text: &str, _span: Span) {}

    /// The [`Buffers::Commands`] buffer had insufficient capacity when trying
    /// to add a [`GCode`].
    fn gcode_buffer_overflowed(
        &mut self,
        _mnemonic: Mnemonic,
        _major_number: u32,
        _minor_number: u32,
        _arguments: &[Word],
        _span: Span,
    ) {
    }

    /// The [`Buffers::Arguments`] buffer had insufficient capacity when trying
    /// to add a [`Word`].
    ///
    /// To aid in diagnostics, the caller is also given the [`GCode`]'s
    /// mnemonic and major/minor numbers.
    fn gcode_argument_buffer_overflowed(
        &mut self,
        _mnemonic: Mnemonic,
        _major_number: u32,
        _minor_number: u32,
        _argument: Word,
    ) {
    }

    /// A [`Comment`] was encountered, but there wasn't enough room in
    /// [`Buffers::Comments`].
    fn comment_buffer_overflow(&mut self, _comment: Comment<'_>) {}

    /// A line number was encountered when it wasn't expected.
    fn unexpected_line_number(&mut self, _line_number: f32, _span: Span) {}

    /// An argument was found, but the parser couldn't figure out which
    /// [`GCode`] it corresponds to.
    fn argument_without_a_command(
        &mut self,
        _letter: char,
        _value: f32,
        _span: Span,
    ) {
    }

    /// A [`Word`]'s number was encountered without an accompanying letter.
    fn number_without_a_letter(&mut self, _value: &str, _span: Span) {}

    /// A [`Word`]'s letter was encountered without an accompanying number.
    fn letter_without_a_number(&mut self, _value: &str, _span: Span) {}
}

impl<'a, C: Callbacks + ?Sized> Callbacks for &'a mut C {
    fn unknown_content(&mut self, text: &str, span: Span) {
        (*self).unknown_content(text, span);
    }

    fn gcode_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        arguments: &[Word],
        span: Span,
    ) {
        (*self).gcode_buffer_overflowed(
            mnemonic,
            major_number,
            minor_number,
            arguments,
            span,
        );
    }

    fn gcode_argument_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        argument: Word,
    ) {
        (*self).gcode_argument_buffer_overflowed(
            mnemonic,
            major_number,
            minor_number,
            argument,
        );
    }

    fn comment_buffer_overflow(&mut self, comment: Comment<'_>) {
        (*self).comment_buffer_overflow(comment);
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        (*self).unexpected_line_number(line_number, span);
    }

    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: Span,
    ) {
        (*self).argument_without_a_command(letter, value, span);
    }

    fn number_without_a_letter(&mut self, value: &str, span: Span) {
        (*self).number_without_a_letter(value, span);
    }

    fn letter_without_a_number(&mut self, value: &str, span: Span) {
        (*self).letter_without_a_number(value, span);
    }
}

/// A set of callbacks that ignore any errors that occur.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct NopCallbacks;

impl Callbacks for NopCallbacks {}
