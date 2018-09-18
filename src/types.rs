use arrayvec::ArrayVec;
use core::cmp;
use core::fmt::{self, Display, Formatter};
use prescaled::{Prescaled, Scalar, TenThousand};

/// The maximum number of arguments a `Gcode` can have.
pub const MAX_ARGS: usize = 8;
type Words = [Word; MAX_ARGS];
/// The type used for all decimal numbers.
pub type Number = Prescaled<TenThousand>;

/// A single command in the `gcode` programming language.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gcode {
    mnemonic: Mnemonic,
    number: Number,
    line_number: Option<u32>,
    // invariant 1: All arguments are uppercase
    arguments: ArrayVec<Words>,
    span: Span,
}

impl Gcode {
    /// Create a new `Gcode`.
    pub fn new2(mnemonic: Mnemonic, number: Number, span: Span) -> Gcode {
        Gcode {
            mnemonic,
            number,
            span,
            arguments: ArrayVec::default(),
            line_number: None,
        }
    }

    /// Create a new `Gcode`.
    pub fn new(mnemonic: Mnemonic, number: f32, span: Span) -> Gcode {
        Gcode::new2(mnemonic, (number as f64).into(), span)
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

    /// Get the line number given to this gode (e.g. the `20` in `N20 G04 P100`).
    pub fn line_number(&self) -> Option<u32> {
        self.line_number
    }

    /// The number associated with this `Gcode` (e.g. the `01` in `G01 X123`).
    #[deprecated = "Use the `major_number` and `minor_number` methods instead"]
    pub fn number(&self) -> f32 {
        f64::from(self.number) as f32
    }

    /// The integral part of the `Gcode`'s number field.
    pub fn major_number(&self) -> u32 {
        self.number.integral_part() as u32
    }

    /// Any number after the decimal point.
    pub fn minor_number(&self) -> Option<u32> {
        let single_digit =
            self.number.fractional_part() * 10 / TenThousand::SCALE;
        if single_digit == 0 {
            None
        } else {
            Some(single_digit as u32)
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
    pub fn value_for(&self, letter: char) -> Option<Number> {
        let letter = letter.to_ascii_uppercase();

        self.arguments
            .iter()
            .find(|word| letter == word.letter)
            .map(|word| word.value)
    }
}

impl Display for Gcode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(n) = self.line_number() {
            write!(f, "N{} ", n)?;
        }

        write!(f, "{}", self.mnemonic())?;
        write!(f, "{}", self.number)?;

        for arg in self.args() {
            write!(f, " {}", arg)?;
        }

        Ok(())
    }
}

/// A single `Word` in the `gcode` language (e.g. `X-12.3`).
#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Word {
    /// The letter associated with this word (e.g. the `X` in `X12.3`).
    pub letter: char,
    /// The numeric part of the word.
    pub value: Number,
    /// The word's location in its original text.
    pub span: Span,
}

impl Word {
    /// Create a new `Word`.
    pub fn new(letter: char, value: Number, span: Span) -> Word {
        Word {
            letter,
            value,
            span,
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.value)
    }
}

/// A general command category.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
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

impl Display for Mnemonic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let c = match *self {
            Mnemonic::ProgramNumber => 'O',
            Mnemonic::ToolChange => 'T',
            Mnemonic::MachineRoutine => 'M',
            Mnemonic::General => 'G',
        };

        write!(f, "{}", c)
    }
}

/// A set of byte indices which correspond to the location of a substring in
/// a larger piece of text.
///
/// The indices are set up such that `&original_text[start .. end]` will yield
/// the selected text.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
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
    pub fn selected_text<'input>(
        &self,
        src: &'input str,
    ) -> Option<&'input str> {
        src.get(self.start..self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_gcode_repr() {
        let thing = Gcode::new(Mnemonic::General, 1.2, Span::default())
            .with_line_number(10, Span::default())
            .with_argument(Word::new('X', 500.0.into(), Span::default()))
            .with_argument(Word::new(
                'Y',
                Number::from(-1.23),
                Span::default(),
            ));
        println!("{:?}", thing);
        let should_be = "N10 G1.2 X500 Y-1.23";

        let got = format!("{}", thing);
        assert_eq!(got, should_be);
    }

    #[test]
    fn you_can_round_trip_a_gcode() {
        let original = Gcode::new2(
            Mnemonic::General,
            Number::from(1.2),
            Span::new(0, 20, 0),
        ).with_line_number(10, Span::default())
        .with_argument(Word::new('X', Number::from(500.0), Span::new(9, 13, 0)))
        .with_argument(Word::new(
            'Y',
            Number::from(-1.23),
            Span::new(14, 20, 0),
        ));

        let serialized = format!("{}", original);

        let got = ::parse(&serialized).next().unwrap();

        assert_eq!(got, original);
    }
}
