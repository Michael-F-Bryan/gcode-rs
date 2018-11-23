mod dwell;
mod linear_interpolate;

pub use self::dwell::Dwell;
pub use self::linear_interpolate::LinearInterpolate;

use core::fmt::{self, Display, Formatter};
use state::State;

pub trait Operation {
    fn state_after(&self, seconds: f32, initial_state: State) -> State;
    fn duration(&self, initial_state: &State) -> f32;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConversionError {
    /// The `Gcode` had the wrong major number.
    IncorrectMajorNumber {
        found: usize,
        expected: &'static [u32],
    },
    /// An argument contained an invalid value.
    InvalidArgument {
        letter: char,
        value: f32,
        message: &'static str,
    },
    /// One or more arguments were missing.
    MissingArguments { expected: &'static [char] },
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ConversionError::MissingArguments { expected } => {
                write!(f, "expected one of the following arguments: ")?;
                write!(f, "{}", expected[0])?;
                for arg in expected.iter().skip(1) {
                    write!(f, ", {}", arg)?;
                }

                Ok(())
            }
            ConversionError::InvalidArgument {
                letter,
                value,
                message,
            } => write!(
                f,
                "The \"{}\" argument has an invalid value of {}, {}",
                letter, value, message
            ),
            ConversionError::IncorrectMajorNumber { found, expected } => {
                write!(f, "The major number {} isn't valid, expected ", found)?;
                if expected.len() > 1 {
                    write!(f, "one of ")?;
                }
                write!(f, "{}", expected[0])?;
                for arg in expected.iter().skip(1) {
                    write!(f, ", {}", arg)?;
                }

                Ok(())
            }
        }
    }
}
