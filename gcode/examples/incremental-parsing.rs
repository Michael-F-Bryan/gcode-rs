//! This example shows how you would use the `gcode` library in a resource
//! constrained environment where all code must use have bounded memory usage
//! (i.e. no dynamic allocation) and the parsing process must periodically yield
//! to allow other tasks to make progress.
//!
//! We drop down to the `gcode::low_level` layer and use the callback mechanism
//! to incrementally fill a buffer with new commands. Parsing is paused when
//! the buffer becomes full or we reach the end of the line, allowing us to
//! limit memory usage to a fixed buffer and make sure we don't spend too much
//! time parsing in one hit.

use arrayvec::ArrayVec;
use gcode::{
    syntax::{Callbacks, Continuation, Parser, ParserState, Word},
    Span,
};
use std::fmt::Debug;

/// Some example gcode taken from
/// https://github.com/dillonhuff/gpr/blob/e1e417e6d724544fe81b26dcf0db38b9fefbb13e/gcode_samples/cura_3D_printer.gcode
const TEXT_TO_PARSE: &str = r#"
;FLAVOR:RepRap
;TIME:713
;Filament used: 0.153994m
;Layer height: 0.1
;Generated with Cura_SteamEngine 2.5.0
M190 S60
M104 S200
M109 S200
G28 ;Home
G1 Z15.0 F6000 ;Move the platform down 15mm
;Prime the extruder
G92 E0
G1 F200 E3
G92 E0
;LAYER_COUNT:58
;LAYER:0
M107
G0 F3600 X43.256 Y45.828 Z0.3
;TYPE:SKIRT
G1 F1800 X43.853 Y45.312 E0.01484
G1 X44.5 Y44.859 E0.0297
G1 X45.15 Y44.493 E0.04373
"#;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut interpreter = Interpreter::default();
    let mut text = TEXT_TO_PARSE;
    let mut parser = Parser::new();

    loop {
        // process as much of the text as we can
        parser.process(text, &mut interpreter)?;

        // do something with the parsed commands
        println!("{:?}", interpreter.words);

        // the commands have been processed so we don't need them any more
        interpreter.words.clear();

        // We probably stopped parsing before reaching the end of our buffer.
        // Update the buffer so we can continue where we left off.
        match interpreter.stopped_at.take() {
            Some(span) => text = &text[span.start..],
            None => break,
        }
    }

    Ok(())
}

/// A simple, incremental G-Code interpreter which will stop at the end of each
/// line or when its word buffer gets full.
#[derive(Debug, Default, Clone, PartialEq)]
struct Interpreter {
    words: ArrayVec<[Word; 4]>,
    stopped_at: Option<Span>,
}

impl Callbacks for Interpreter {
    type Error = core::convert::Infallible;

    fn on_word(
        &mut self,
        span: Span,
        word: Word,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        if self.words.try_push(word).is_err() {
            // looks like we ran out of buffer space, remember the current
            // position and bail.
            self.stopped_at = Some(span);
            Continuation::Break
        } else {
            Continuation::Continue
        }
    }

    fn on_end_line(
        &mut self,
        span: Span,
        _line: &str,
        _state: &ParserState,
    ) -> Continuation<Self::Error> {
        // we want to stop at the end of each line (e.g. to make sure we don't
        // spend our full time budget parsing)
        self.stopped_at = Some(span);
        Continuation::Break
    }
}
