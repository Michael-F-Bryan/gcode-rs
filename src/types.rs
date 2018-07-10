use arrayvec::ArrayVec;
use core::cmp;

pub type Words = [Word; 8];

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gcode {
    pub mnemonic: Mnemonic,
    pub number: f32,
    pub arguments: ArrayVec<Words>,
    pub span: Span,
}

impl Gcode {
    pub fn new(mnemonic: Mnemonic, number: f32, span: Span) -> Gcode {
        Gcode {
            mnemonic,
            number,
            span,
            arguments: ArrayVec::new(),
        }
    }

    pub fn add_argument(&mut self, arg: Word) {
        self.span = self.span.merge(&arg.span);

        match self.arguments.iter().position(|w| w.letter == arg.letter) {
            Some(i) => self.arguments[i] = arg,
            None => {
                let _ = self.arguments.try_push(arg);
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Word {
    pub span: Span,
    pub letter: char,
    pub number: f32,
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
