use arrayvec::ArrayVec;
use core::cmp;

/// The maximum number of arguments a `Gcode` can have.
pub const MAX_ARGS: usize = 8;
type Words = [Word; MAX_ARGS];

/// A single command in the `gcode` programming language.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gcode {
    mnemonic: Mnemonic,
    number: f32,
    line_number: Option<u32>,
    // invariant 1: All arguments are uppercase
    arguments: ArrayVec<Words>,
    span: Span,
}

impl Gcode {
    /// Create a new `Gcode`.
    pub fn new(mnemonic: Mnemonic, number: f32, span: Span) -> Gcode {
        Gcode {
            mnemonic,
            number,
            span,
            arguments: ArrayVec::default(),
            line_number: None,
        }
    }

    /// Get the `Mnemonic` used by this `Gcode`.
    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }

    /// Get the location of this `Gcode` in the original text.
    pub fn span(&self) -> Span {
        self.span
    }

    /// The arguments provided to the `Gcode`.
    pub fn args(&self) -> &[Word] {
        &self.arguments
    }

    /// The number associated with this `Gcode` (e.g. the `01` in `G01 X123`).
    pub fn number(&self) -> f32 {
        self.number
    }

    /// The integral part of the `Gcode`'s number field.
    pub fn major_number(&self) -> u32 {
        self.number.trunc() as u32
    }

    /// Any number after the decimal point.
    pub fn minor_number(&self) -> Option<u32> {
        let remainder = self.number.fract();

        if remainder == 0.0 {
            None
        } else {
            unimplemented!()
        }
    }

    fn merge_span(&mut self, span: Span) {
        self.span = self.span.merge(span);
    }

    /// Add an argument to this `Gcode`'s argument list.
    pub fn add_argument(&mut self, mut arg: Word) {
        self.merge_span(arg.span);
        arg.letter = arg.letter.to_ascii_uppercase();

        match self.arguments.iter().position(|w| w.letter == arg.letter) {
            Some(i) => self.arguments[i] = arg,
            None => {
                let _ = self.arguments.try_push(arg);
            }
        }
    }

    /// A builder method for adding an argument to the `Gcode`.
    pub fn with_argument(mut self, arg: Word) -> Self {
        self.add_argument(arg);
        self
    }

    /// A builder method for attaching a line number (the `30` in `N30 G01 X32`)
    /// to a command.
    pub fn with_line_number(mut self, number: u32, span: Span) -> Self {
        self.merge_span(span);
        self.line_number = Some(number);

        self
    }

    /// Find the value for the desired argument.
    pub fn value_for(&self, letter: char) -> Option<f32> {
        let letter = letter.to_ascii_uppercase();

        self.arguments.iter()
            .find(|word| letter == word.letter)
            .map(|word| word.value)
    }
}

/// A single `Word` in the `gcode` language (e.g. `X-12.3`).
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Word {
    /// The letter associated with this word (e.g. the `X` in `X12.3`).
    pub letter: char,
    /// The numeric part of the word.
    pub value: f32,
    /// The word's location in its original text.
    pub span: Span,
}

impl Word {
    /// Create a new `Word`.
    pub fn new(letter: char, value: f32, span: Span) -> Word {
        Word { letter, value, span }
    }
}

/// A general command category.
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

/// A set of byte indices which correspond to the location of a substring in
/// a larger piece of text.
///
/// The indices are set up such that `&original_text[start .. end]` will yield
/// the selected text.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// The starting index.
    pub start: usize,
    /// The index **one past the end** of the selected text.
    pub end: usize,
    /// Which line (zero indexed) does the text start on?
    pub source_line: usize,
}

impl Span {
    /// Create a new `Span`.
    pub fn new(start: usize, end: usize, source_line: usize) -> Span {
        debug_assert!(start <= end);
        Span {
            start,
            end,
            source_line,
        }
    }

    /// Get the union of two spans.
    pub fn merge(&self, other: Span) -> Span {
        Span {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
            source_line: cmp::min(self.source_line, other.source_line),
        }
    }

    /// The number of bytes within this span.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Given the original text, retrieve the substring this `Span` corresponds
    /// to.
    pub fn selected_text<'input>(&self, src: &'input str) -> Option<&'input str> {
        src.get(self.start..self.end)
    }
}
