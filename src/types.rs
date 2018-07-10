use arrayvec::ArrayVec;
use core::cmp;

pub const MAX_ARGS: usize = 8;
pub type Words = [Word; MAX_ARGS];

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gcode {
    pub(crate) mnemonic: Mnemonic,
    pub(crate) number: f32,
    pub(crate) line_number: Option<Word>,
    pub(crate) arguments: ArrayVec<Words>,
    pub(crate) span: Span,
}

impl Gcode {
    pub fn new<F: Into<f32>>(mnemonic: Mnemonic, number: F, span: Span) -> Gcode {
        Gcode {
            mnemonic,
            number: number.into(),
            span,
            arguments: ArrayVec::default(),
            line_number: None,
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }

    pub fn args(&self) -> &[Word] {
        &self.arguments
    }

    pub fn major_number(&self) -> u32 {
        self.number.trunc() as u32
    }

    pub fn minor_number(&self) -> Option<u32> {
        let remainder = self.number.fract();
        unimplemented!()
    }

    fn merge_span(&mut self, span: &Span) {
        self.span = self.span.merge(span);
    }

    pub fn add_argument(&mut self, arg: Word) {
        self.merge_span(&arg.span);

        match self.arguments.iter().position(|w| w.letter == arg.letter) {
            Some(i) => self.arguments[i] = arg,
            None => {
                let _ = self.arguments.try_push(arg);
            }
        }
    }

    pub fn with_argument(mut self, arg: Word) -> Self {
        self.add_argument(arg);
        self
    }

    pub fn with_line_number(mut self, number: Word) -> Self {
        self.merge_span(&number.span);
        self.line_number = Some(number);

        self
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Word {
    pub span: Span,
    pub letter: char,
    pub number: f32,
}

impl Word {
    pub fn new<F: Into<f32>>(letter: char, number: F, span: Span) -> Word {
        Word { letter, number: number.into(), span }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mnemonic {
    /// A program number (`O555`).
    ProgramNumber,
    /// A tool change command (`T6`).
    ToolChange,
    /// A machine-specific routine (`M3`).
    MachineRoutine,
    /// A general command (`G01`).
    General,
}

impl Default for Mnemonic {
    fn default() -> Mnemonic {
        Mnemonic::General
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source_line: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, source_line: usize) -> Span {
        debug_assert!(start <= end);
        Span {
            start,
            end,
            source_line,
        }
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
            source_line: cmp::min(self.source_line, other.source_line),
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn selected_text<'input>(&self, src: &'input str) -> Option<&'input str> {
        src.get(self.start..self.end)
    }
}
