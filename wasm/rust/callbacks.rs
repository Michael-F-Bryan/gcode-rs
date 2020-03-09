use crate::{Span, Word};
use gcode::{Callbacks, Mnemonic};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    pub type JavaScriptCallbacks;

    #[wasm_bindgen(method)]
    fn unknown_content(this: &JavaScriptCallbacks, text: &str, span: Span);

    #[wasm_bindgen(method)]
    fn gcode_buffer_overflowed(
        this: &JavaScriptCallbacks,
        mnemonic: char,
        major_number: u32,
        minor_number: u32,
        span: Span,
    );

    #[wasm_bindgen(method)]
    fn gcode_argument_buffer_overflowed(
        this: &JavaScriptCallbacks,
        mnemonic: char,
        major_number: u32,
        minor_number: u32,
        argument: Word,
    );

    #[wasm_bindgen(method)]
    fn comment_buffer_overflow(
        this: &JavaScriptCallbacks,
        comment: &str,
        span: Span,
    );

    #[wasm_bindgen(method)]
    fn unexpected_line_number(
        this: &JavaScriptCallbacks,
        line_number: f32,
        span: Span,
    );

    #[wasm_bindgen(method)]
    fn argument_without_a_command(
        this: &JavaScriptCallbacks,
        letter: char,
        value: f32,
        span: Span,
    );

    #[wasm_bindgen(method)]
    fn number_without_a_letter(
        this: &JavaScriptCallbacks,
        value: &str,
        span: Span,
    );

    #[wasm_bindgen(method)]
    fn letter_without_a_number(
        this: &JavaScriptCallbacks,
        value: &str,
        span: Span,
    );
}

impl Callbacks for JavaScriptCallbacks {
    fn unknown_content(&mut self, text: &str, span: gcode::Span) {
        JavaScriptCallbacks::unknown_content(self, text, span.into());
    }

    fn gcode_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        _arguments: &[gcode::Word],
        span: gcode::Span,
    ) {
        JavaScriptCallbacks::gcode_buffer_overflowed(
            self,
            crate::mnemonic_letter(mnemonic),
            major_number,
            minor_number,
            span.into(),
        );
    }

    fn gcode_argument_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        argument: gcode::Word,
    ) {
        JavaScriptCallbacks::gcode_argument_buffer_overflowed(
            self,
            crate::mnemonic_letter(mnemonic),
            major_number,
            minor_number,
            argument.into(),
        );
    }

    fn comment_buffer_overflow(&mut self, comment: gcode::Comment) {
        JavaScriptCallbacks::comment_buffer_overflow(
            self,
            comment.value,
            comment.span.into(),
        );
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: gcode::Span) {
        JavaScriptCallbacks::unexpected_line_number(
            self,
            line_number,
            span.into(),
        );
    }

    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: gcode::Span,
    ) {
        JavaScriptCallbacks::argument_without_a_command(
            self,
            letter,
            value,
            span.into(),
        );
    }

    fn number_without_a_letter(&mut self, value: &str, span: gcode::Span) {
        JavaScriptCallbacks::number_without_a_letter(self, value, span.into());
    }

    fn letter_without_a_number(&mut self, value: &str, span: gcode::Span) {
        JavaScriptCallbacks::letter_without_a_number(self, value, span.into());
    }
}
