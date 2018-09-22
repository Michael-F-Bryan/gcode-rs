use arrayvec::ArrayVec;
#[cfg(not(feature = "std"))]
use libm::F32Ext;

const INLINE_COMMENT_COUNT: usize = 3;
const ARGUMENT_COUNT: usize = 10;
type Arguments = ArrayVec<[Argument; ARGUMENT_COUNT]>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source_line: usize,
}

impl Span {
    pub fn placeholder() -> Span {
        Span {
            start: usize::max_value(),
            end: usize::max_value(),
            source_line: usize::max_value(),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        *self == Span::placeholder()
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::placeholder()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block<'input> {
    src: &'input str,
    span: Span,
}

/// A single gcode command.
#[derive(Debug, Clone, PartialEq)]
pub struct Gcode<'input> {
    line_number: Option<usize>,
    mnemonic: Mnemonic,
    number: f32,
    comments: ArrayVec<[Comment<'input>; INLINE_COMMENT_COUNT]>,
    arguments: Arguments,
    span: Span,
}

impl<'input> Gcode<'input> {
    pub fn new(mnemonic: Mnemonic, number: f32) -> Gcode<'input> {
        debug_assert!(number > 0.0, "The number should always be positive");
        Gcode {
            mnemonic,
            number: number as f32,
            line_number: None,
            comments: Default::default(),
            arguments: Default::default(),
            span: Default::default(),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }

    pub fn major_number(&self) -> usize {
        debug_assert!(
            self.number >= 0.0,
            "The number should always be positive"
        );
        self.number.trunc() as usize
    }

    pub fn minor_number(&self) -> Option<usize> {
        debug_assert!(
            self.number >= 0.0,
            "The number should always be positive"
        );
        let digit = (self.number.fract() / 0.1).round();

        if digit == 0.0 {
            None
        } else {
            Some(digit as usize)
        }
    }

    pub fn args(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn args_mut(&mut self) -> &mut [Argument] {
        &mut self.arguments
    }

    pub fn with_line_number(&mut self, number: usize) -> &mut Self {
        self.line_number = Some(number);
        self
    }

    pub fn with_minor_nujmber(&mut self, number: usize) -> &mut Self {
        debug_assert!(number < 10);
        self.number = self.number.trunc() + number as f32 / 10.0;
        self
    }

    pub fn with_comment(&mut self, comment: Comment<'input>) -> &mut Self {
        self.comments.push(comment);
        self
    }

    pub fn with_argument(&mut self, arg: Argument) -> &mut Self {
        self.arguments.push(arg);
        self
    }

    pub fn with_span(&mut self, span: Span) -> &mut Self {
        self.span = span;
        self
    }
}

/// A command argument.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Argument {
    pub letter: char,
    pub value: f32,
    pub span: Span,
}

/// A comment.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Comment<'input> {
    pub body: &'input str,
    pub span: Span,
}

/// The general category a `Gcode` belongs to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Mnemonic {
    General,
    Miscellaneous,
    ProgramNumber,
    ToolChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_extract_major_and_minor_number() {
        let input = vec![
            (1.0, 1, None),
            (1.1, 1, Some(1)),
            (1.2, 1, Some(2)),
            (1.3, 1, Some(3)),
            (1.4, 1, Some(4)),
            (1.5, 1, Some(5)),
            (1.6, 1, Some(6)),
            (2.7, 2, Some(7)),
        ];

        for (number, major, minor) in input {
            let g = Gcode::new(Mnemonic::General, number);

            assert_eq!(g.major_number(), major);
            assert_eq!(g.minor_number(), minor);
        }
    }
}
