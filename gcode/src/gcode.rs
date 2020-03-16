use crate::{
    buffers::{Buffer, CapacityError, DefaultArguments},
    Span, Word,
};
use core::fmt::{self, Debug, Display, Formatter};

/// The general category for a [`GCode`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
#[repr(C)]
pub enum Mnemonic {
    /// Preparatory commands, often telling the controller what kind of motion
    /// or offset is desired.
    General,
    /// Auxilliary commands.
    Miscellaneous,
    /// Used to give the current program a unique "name".
    ProgramNumber,
    /// Tool selection.
    ToolChange,
}

impl Mnemonic {
    /// Try to convert a letter to its [`Mnemonic`] equivalent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gcode::Mnemonic;
    /// assert_eq!(Mnemonic::for_letter('M'), Some(Mnemonic::Miscellaneous));
    /// assert_eq!(Mnemonic::for_letter('g'), Some(Mnemonic::General));
    /// ```
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

/// The in-memory representation of a single command in the G-code language
/// (e.g. `"G01 X50.0 Y-20.0"`).
#[derive(Clone)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
pub struct GCode<A = DefaultArguments> {
    mnemonic: Mnemonic,
    number: f32,
    arguments: A,
    span: Span,
}

impl GCode {
    /// Create a new [`GCode`] which uses the [`DefaultArguments`] buffer.
    pub fn new(mnemonic: Mnemonic, number: f32, span: Span) -> Self {
        GCode {
            mnemonic,
            number,
            span,
            arguments: DefaultArguments::default(),
        }
    }
}

impl<A: Buffer<Word>> GCode<A> {
    /// Create a new [`GCode`] which uses a custom [`Buffer`].
    pub fn new_with_argument_buffer(
        mnemonic: Mnemonic,
        number: f32,
        span: Span,
        arguments: A,
    ) -> Self {
        GCode {
            mnemonic,
            number,
            span,
            arguments,
        }
    }

    /// The overall category this [`GCode`] belongs to.
    pub fn mnemonic(&self) -> Mnemonic { self.mnemonic }

    /// The integral part of a command number (i.e. the `12` in `G12.3`).
    pub fn major_number(&self) -> u32 {
        debug_assert!(self.number >= 0.0);

        libm::floorf(self.number) as u32
    }

    /// The fractional part of a command number (i.e. the `3` in `G12.3`).
    pub fn minor_number(&self) -> u32 {
        let fract = self.number - libm::floorf(self.number);
        let digit = libm::roundf(fract * 10.0);
        digit as u32
    }

    /// The arguments attached to this [`GCode`].
    pub fn arguments(&self) -> &[Word] { self.arguments.as_slice() }

    /// Where the [`GCode`] was found in its source text.
    pub fn span(&self) -> Span { self.span }

    /// Add an argument to the list of arguments attached to this [`GCode`].
    pub fn push_argument(
        &mut self,
        arg: Word,
    ) -> Result<(), CapacityError<Word>> {
        self.span = self.span.merge(arg.span);
        self.arguments.try_push(arg)
    }

    /// The builder equivalent of [`GCode::push_argument()`].
    ///
    /// # Panics
    ///
    /// This will panic if the underlying [`Buffer`] returns a
    /// [`CapacityError`].
    pub fn with_argument(mut self, arg: Word) -> Self {
        if let Err(e) = self.push_argument(arg) {
            panic!("Unable to add the argument {:?}: {}", arg, e);
        }
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

impl<A: Buffer<Word>> Extend<Word> for GCode<A> {
    fn extend<I: IntoIterator<Item = Word>>(&mut self, words: I) {
        for word in words {
            if self.push_argument(word).is_err() {
                // we can't add any more arguments
                return;
            }
        }
    }
}

impl<A: Buffer<Word>> Debug for GCode<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // we manually implement Debug because the the derive will constrain
        // the buffer type to be Debug, which isn't necessary and actually makes
        // it impossible to print something like ArrayVec<[T; 128]>
        let GCode {
            mnemonic,
            number,
            arguments,
            span,
        } = self;

        f.debug_struct("GCode")
            .field("mnemonic", mnemonic)
            .field("number", number)
            .field("arguments", &crate::buffers::debug(arguments))
            .field("span", span)
            .finish()
    }
}

impl<A: Buffer<Word>> Display for GCode<A> {
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

impl<A, B> PartialEq<GCode<B>> for GCode<A>
where
    A: Buffer<Word>,
    B: Buffer<Word>,
{
    fn eq(&self, other: &GCode<B>) -> bool {
        let GCode {
            mnemonic,
            number,
            arguments,
            span,
        } = self;

        *span == other.span()
            && *mnemonic == other.mnemonic
            && *number == other.number
            && arguments.as_slice() == other.arguments.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;
    use std::prelude::v1::*;

    type BigBuffer = ArrayVec<[Word; 32]>;

    #[test]
    fn correct_major_number() {
        let code = GCode {
            mnemonic: Mnemonic::General,
            number: 90.5,
            arguments: BigBuffer::default(),
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
                arguments: BigBuffer::default(),
                span: Span::default(),
            };

            assert_eq!(code.minor_number(), i);
        }
    }

    #[test]
    fn get_argument_values() {
        let mut code = GCode::new_with_argument_buffer(
            Mnemonic::General,
            90.0,
            Span::default(),
            BigBuffer::default(),
        );
        code.push_argument(Word {
            letter: 'X',
            value: 10.0,
            span: Span::default(),
        })
        .unwrap();
        code.push_argument(Word {
            letter: 'y',
            value: -3.5,
            span: Span::default(),
        })
        .unwrap();

        assert_eq!(code.value_for('X'), Some(10.0));
        assert_eq!(code.value_for('x'), Some(10.0));
        assert_eq!(code.value_for('Y'), Some(-3.5));
        assert_eq!(code.value_for('Z'), None);
    }
}
