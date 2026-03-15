use core::fmt::{self, Display, Formatter};

pub type ControlFlow<T> = core::ops::ControlFlow<(), T>;

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
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

pub trait ProgramVisitor {
    fn start_line(&mut self, span: Span) -> ControlFlow<impl LineVisitor + '_>;
}

pub trait LineVisitor {
    fn line_number(&mut self, _n: f32, _span: Span) {}
    fn comment(&mut self, _value: &str, _span: Span) {}
    fn program_number(&mut self, _number: Number, _span: Span) {}

    fn start_general_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_miscellaneous_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_tool_change_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }

    fn unknown_content_error(&mut self, _text: &str, _span: Span) {}
    fn unexpected_line_number_error(&mut self, _n: f32, _span: Span) {}
    fn letter_without_number_error(&mut self, _value: &str, _span: Span) {}
    fn number_without_letter_error(&mut self, _value: &str, _span: Span) {}

    /// Called at the end of the line to finalize the line visitor.
    fn end_line(&mut self) {}
}

pub trait CommandVisitor {
    fn argument(&mut self, _letter: char, _value: f32, _span: Span) {}

    fn argument_buffer_overflow_error(
        &mut self,
        _letter: char,
        _value: f32,
        _span: Span,
    ) {
    }

    /// Called at the end of the command scope to finalize the command visitor.
    fn end_command(&mut self) {}
}

struct Noop;

impl CommandVisitor for Noop {}

impl LineVisitor for Noop {}
