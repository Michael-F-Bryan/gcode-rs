use crate::{Span, Word};
use core::fmt::{self, Display, Formatter};

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        type Arguments = Vec<Word>;
    } else {
        use arrayvec::ArrayVec;
        type Arguments = ArrayVec<[Word; GCode::MAX_ARGUMENT_LEN]>;
    }
}

/// The general category for a [`GCode`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Mnemonic {
    General,
    Miscellaneous,
    ProgramNumber,
    ToolChange,
}

impl Mnemonic {
    pub fn for_letter(letter: char) -> Option<Mnemonic> {
        match letter.to_ascii_lowercase() {
            'g' => Some(Mnemonic::General),
            'm' => Some(Mnemonic::Miscellaneous),
            'o' => Some(Mnemonic::ProgramNumber),
            't' => Some(Mnemonic::ToolChange),
            _ => None,
        }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mnemonic::General => write!(f, "G"),
            Mnemonic::Miscellaneous => write!(f, "M"),
            Mnemonic::ProgramNumber => write!(f, "O"),
            Mnemonic::ToolChange => write!(f, "T"),
        }
    }
}

/// A single gcode command.
#[derive(Debug, Clone, PartialEq)]
pub struct GCode {
    mnemonic: Mnemonic,
    number: f32,
    arguments: Arguments,
    span: Span,
}

impl GCode {
    /// The maximum number of [`Word`]s when compiled without the `std`
    /// feature.
    pub const MAX_ARGUMENT_LEN: usize = 5;

    pub fn new(mnemonic: Mnemonic, number: f32, span: Span) -> Self {
        GCode {
            mnemonic,
            number,
            span,
            arguments: Arguments::default(),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic { self.mnemonic }

    pub fn major_number(&self) -> u32 {
        debug_assert!(self.number >= 0.0);

        self.number.floor() as u32
    }

    pub fn minor_number(&self) -> u32 {
        let fract = self.number - self.number.floor();
        let digit = (fract * 10.0).round();
        digit as u32
    }

    pub fn arguments(&self) -> &[Word] { &self.arguments }

    pub fn span(&self) -> Span { self.span }

    /// Add an argument to the list of arguments attached to this [`GCode`].
    pub fn push_argument(&mut self, arg: Word) { self.arguments.push(arg); }

    /// The builder equivalent of [`GCode::push_argument()`].
    pub fn with_argument(mut self, arg: Word) -> Self {
        self.push_argument(arg);
        self
    }

    /// Get the value for a particular argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gcode::{GCode, Mnemonic, Span, Word};
    /// let gcode = GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
    ///     .with_argument(Word::new('X', 30.0, Span::PLACEHOLDER))
    ///     .with_argument(Word::new('Y', -3.14, Span::PLACEHOLDER));
    ///
    /// assert_eq!(gcode.value_for('Y'), Some(-3.14));
    /// ```
    pub fn value_for(&self, letter: char) -> Option<f32> {
        let letter = letter.to_ascii_lowercase();

        for arg in self.arguments() {
            if arg.letter.to_ascii_lowercase() == letter {
                return Some(arg.value);
            }
        }

        None
    }
}

impl Extend<Word> for GCode {
    fn extend<I: IntoIterator<Item = Word>>(&mut self, words: I) {
        for word in words {
            self.push_argument(word);
        }
    }
}

impl Display for GCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.mnemonic(), self.major_number())?;

        if self.minor_number() != 0 {
            write!(f, ".{}", self.minor_number())?;
        }

        for arg in self.arguments() {
            write!(f, " {}", arg)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_major_number() {
        let code = GCode {
            mnemonic: Mnemonic::General,
            number: 90.5,
            arguments: Arguments::default(),
            span: Span::default(),
        };

        assert_eq!(code.major_number(), 90);
    }

    #[test]
    fn correct_minor_number() {
        for i in 0..=9 {
            let code = GCode {
                mnemonic: Mnemonic::General,
                number: 10.0 + (i as f32) / 10.0,
                arguments: Arguments::default(),
                span: Span::default(),
            };
            println!("{:?}", code);

            assert_eq!(code.minor_number(), i);
        }
    }

    #[test]
    fn get_argument_values() {
        let mut code = GCode::new(Mnemonic::General, 90.0, Span::default());
        code.extend(vec![
            Word {
                letter: 'X',
                value: 10.0,
                span: Span::default(),
            },
            Word {
                letter: 'y',
                value: -3.14,
                span: Span::default(),
            },
        ]);

        assert_eq!(code.value_for('X'), Some(10.0));
        assert_eq!(code.value_for('x'), Some(10.0));
        assert_eq!(code.value_for('Y'), Some(-3.14));
        assert_eq!(code.value_for('Z'), None);
    }
}
