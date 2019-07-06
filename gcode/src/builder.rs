#![allow(missing_docs)]

use crate::{Comment, Gcode};
use core::fmt::Write;

pub trait Sink {
    type Error;

    fn emit_gcode(&mut self, gcode: &Gcode) -> Result<(), Self::Error>;
    fn emit_comment(&mut self, comment: &Comment<'_>) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyWriterSink<W> {
    writer: W,
}

impl<W: Write> PrettyWriterSink<W> {
    pub fn new(writer: W) -> PrettyWriterSink<W> {
        PrettyWriterSink { writer }
    }
}

impl<W: Write> Sink for PrettyWriterSink<W> {
    type Error = core::fmt::Error;

    fn emit_gcode(&mut self, gcode: &Gcode) -> Result<(), Self::Error> {
        if let Some(n) = gcode.line_number() {
            writeln!(self.writer, "N{} ", n)?;
        }

        writeln!(self.writer, "{}{}", gcode.mnemonic(), gcode.major_number())?;
        if let Some(minor) = gcode.minor_number() {
            writeln!(self.writer, ".{}", minor)?;
        }

        for arg in gcode.args() {
            writeln!(self.writer, " {}{}", arg.letter, arg.value)?;
        }

        Ok(())
    }

    fn emit_comment(&mut self, comment: &Comment<'_>) -> Result<(), Self::Error> {
        writeln!(self.writer, "; {}", comment.body())
    }
}
