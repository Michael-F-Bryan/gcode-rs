use crate::{JavaScriptCallbacks, Line};
use std::{mem::ManuallyDrop, pin::Pin};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Parser {
    /// A pinned heap allocation containing the text that `inner` has
    /// references to.
    ///
    /// # Safety
    ///
    /// This field **must** be destroyed after `inner`. The `&str` it contains
    /// should also never change, otherwise we may invalidate references inside
    /// `inner`.
    _text: Pin<Box<str>>,
    /// The actual `gcode::Parser`. We've told the compiler that it has a
    /// `'static` lifetime because we'll be using `unsafe` code to manually
    /// manage memory.
    inner: ManuallyDrop<gcode::Parser<'static, JavaScriptCallbacks>>,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new(text: String, callbacks: JavaScriptCallbacks) -> Parser {
        // make sure our text is allocated on the heap and will never move
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
            // get a pointer to the underlying text
            let text_ptr: *const str = &*text;
            // then convert it to a reference with a 'static lifetime
            let static_str: &'static str = &*text_ptr;

            // now make a gcode::Parser which uses the 'static text as input
            let inner =
                ManuallyDrop::new(gcode::Parser::new(static_str, callbacks));

            Parser { _text: text, inner }
        }
    }

    /// Try to parse the next [`Line`].
    ///
    /// # Safety
    ///
    /// The line must not outlive the [`Parser`] it came from.
    pub fn next_line(&mut self) -> Option<Line> { self.inner.next().map(From::from) }
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
