use std::{mem::ManuallyDrop, pin::Pin};
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

#[wasm_bindgen]
pub struct Span(gcode::Span);

#[wasm_bindgen]
impl Span {
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> usize { self.0.start }

    #[wasm_bindgen(getter)]
    pub fn end(&self) -> usize { self.0.end }

    #[wasm_bindgen(getter)]
    pub fn line(&self) -> usize { self.0.line }
}

#[wasm_bindgen]
pub struct Word(gcode::Word);

#[wasm_bindgen]
impl Word {
    #[wasm_bindgen(getter)]
    pub fn letter(&self) -> char { self.0.letter }

    #[wasm_bindgen(getter)]
    pub fn span(&self) -> Span { Span(self.0.span) }
}

#[wasm_bindgen]
pub struct Parser {
    _text: Pin<Box<str>>,
    inner: ManuallyDrop<gcode::Parser<'static, JavaScriptCallbacks>>,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new(text: String, callbacks: JavaScriptCallbacks) -> Parser {
        let text: Pin<Box<str>> = text.into_boxed_str().into();

        // SAFETY: Because gcode::Parser contains a reference to the text string
        // it needs a lifetime, however it's not sound to expose a struct with
        // a lifetime to JavaScript (JavaScript doesn't have a borrow checker).
        //
        // To work around this we turn the string into a `Box<Pin<str>>` (to
        // statically ensure pointers to our string will never change) then
        // take a reference to it and "extend" the reference lifetime to
        // 'static.
        //
        // The order that `text` and `inner` are destroyed in isn't really
        // defined, so we use `ManuallyDrop` to ensure the `gcode::Parser` is
        // destroyed first. That way we don't get the situation where `text` is
        // destroyed and our `inner` parser is left with dangling references.
        unsafe {
            let text_ptr: *const str = &*text;
            let static_str: &'static str = &*text_ptr;

            let inner =
                ManuallyDrop::new(gcode::Parser::new(static_str, callbacks));

            Parser { _text: text, inner }
        }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        // SAFETY: This is the only place `inner` gets destroyed, and the field
        // can never be touch after `Parser::drop()` is called.
        unsafe {
            ManuallyDrop::drop(&mut self.inner);
        }

        // the text will be destroyed somewhere after here because Rust's drop
        // glue destroys fields after the containing type is destroyed.
    }
}
