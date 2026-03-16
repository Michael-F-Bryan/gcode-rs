use core::fmt::{self, Display, Formatter};

pub type ControlFlow<T> = core::ops::ControlFlow<(), T>;

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[repr(C)]
pub struct Span {
    /// The byte index corresponding to the item's start.
    pub start: usize,
    /// The number of bytes in the span.
    pub length: usize,
    /// The (zero-based) line number.
    pub line: usize,
}

impl Span {
    /// Constructs a span from start index, length in bytes, and line number.
    pub const fn new(start: usize, length: usize, line: usize) -> Self {
        Self {
            start,
            length,
            line,
        }
    }

    /// The index one byte past the item's end.
    #[must_use]
    pub const fn end(self) -> usize {
        self.start + self.length
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Number {
    pub major: u16,
    pub minor: Option<u16>,
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Number { major, minor } = self;
        write!(f, "{}", major)?;
        if let Some(minor) = minor {
            write!(f, ".{}", minor)?;
        }
        Ok(())
    }
}

/// A set of callbacks used to report errors.
#[allow(unused_variables)]
pub trait Diagnostics {
    fn emit_unknown_content(&mut self, text: &str, span: Span) {}

    fn emit_unexpected(&mut self, actual: &str, expected: &[&str], span: Span) {
    }
}

pub trait HasDiagnostics {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics;
}

impl<D: Diagnostics> HasDiagnostics for D {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics {
        &mut *self
    }
}

pub trait ProgramVisitor: HasDiagnostics {
    fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_>;
}

#[allow(unused_variables)]
pub trait BlockVisitor: HasDiagnostics + Sized {
    fn line_number(&mut self, n: Number, span: Span) {}
    fn comment(&mut self, value: &str, span: Span) {}
    fn program_number(&mut self, number: Number, span: Span) {}

    fn start_general_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_miscellaneous_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_tool_change_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }

    /// Called at the end of the line to finalize the line visitor.
    fn end_line(self, span: Span) {}
}

#[allow(unused_variables)]
pub trait CommandVisitor: HasDiagnostics + Sized {
    fn argument(&mut self, letter: char, value: Value<'_>, span: Span) {}

    /// Called at the end of the command scope to finalize the command visitor.
    fn end_command(self, span: Span) {}
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Value<'src> {
    Literal(f32),
    Variable(&'src str),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Noop;

impl ProgramVisitor for Noop {
    fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
}

impl CommandVisitor for Noop {}

impl BlockVisitor for Noop {}

impl Diagnostics for Noop {}
