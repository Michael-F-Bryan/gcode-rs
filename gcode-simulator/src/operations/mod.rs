mod dwell;
mod linear_interpolate;

pub use self::dwell::Dwell;
pub use self::linear_interpolate::LinearInterpolate;

use crate::TryFrom;
use core::fmt::{self, Display, Formatter};
use gcode::Gcode;
use state::State;
use sum_type;
use uom::si::f32::Time;

pub trait Operation {
    fn state_after(&self, seconds: Time, initial_state: State) -> State;
    fn duration(&self, initial_state: &State) -> Time;
}

/// A helper trait for anything which a `Gcode` *might* be able to transform
/// into.
pub trait FromGcode: TryFrom<Gcode, Error = ConversionError> {
    fn valid_major_numbers() -> &'static [usize];
}

sum_type::sum_type! {
    /// A union of all known operations.
    #[derive(Copy, Debug, Clone, PartialEq)]
    pub enum Op {
        Dwell,
        LinearInterpolate,
    }
}

impl TryFrom<Gcode> for Op {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        let major = other.major_number();

        macro_rules! maybe_convert {
            ($($variant:ident),*) => {
                $(
                    if $variant::valid_major_numbers().contains(&major) {
                        return $variant::try_from(other).map(Into::into);
                    }
                )*
            };
        }

        maybe_convert!(Dwell, LinearInterpolate);

        Err(ConversionError::IncorrectMajorNumber {
            found: other.major_number(),
            expected: Op::valid_major_numbers(),
        })
    }
}

impl FromGcode for Op {
    fn valid_major_numbers() -> &'static [usize] {
        &[0, 1, 4]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConversionError {
    /// The `Gcode` had the wrong major number.
    IncorrectMajorNumber {
        found: usize,
        expected: &'static [usize],
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::prelude::v1::*;
    use sum_type::SumType;

    fn variant_count() -> usize {
        Op::Dwell(Dwell::from_seconds(0.0)).variants().len()
    }

    #[test]
    fn op_valid_major_number_is_in_sync() {
        let count = variant_count();
        let should_be = vec![
            Dwell::valid_major_numbers(),
            LinearInterpolate::valid_major_numbers(),
        ];
        // make sure our should_be vector is correct
        assert_eq!(
            should_be.len(),
            count,
            "There should be items from {} variants",
            count
        );
        let major_number_count: usize =
            should_be.iter().map(|slice| slice.len()).sum();
        let should_be: HashSet<_> =
            should_be.into_iter().flatten().cloned().collect();

        let got = Op::valid_major_numbers();
        assert_eq!(got.len(), major_number_count);

        let got: HashSet<_> = got.into_iter().cloned().collect();
        assert_eq!(got, should_be);
    }
}
