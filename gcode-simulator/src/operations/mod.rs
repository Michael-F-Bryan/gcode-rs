//! Translations from low level `Gcode`s to a more machine-friendly form.

#[macro_use]
mod helpers;
mod coordinate_modes;
mod dwell;
mod linear_interpolate;
mod units;

pub use self::coordinate_modes::{AbsoluteCoordinates, RelativeCoordinates};
pub use self::dwell::Dwell;
pub use self::linear_interpolate::LinearInterpolate;
pub use self::units::{Imperial, Metric};

use crate::TryFrom;
use gcode::Gcode;
use state::State;
use std::fmt::{self, Display, Formatter};
use sum_type;
use uom::si::f32::Time;

/// A gcode operation.
pub trait Operation {
    /// Evaluate the machine state `delta` time after the operation started.
    fn state_after(&self, delta: Time, initial_state: State) -> State;
    /// How long will this `Operation` take to complete?
    fn duration(&self, initial_state: &State) -> Time;
}

/// A helper trait for anything which a `Gcode` *might* be able to transform
/// into.
pub trait FromGcode<'a>: TryFrom<&'a Gcode, Error = ConversionError> {
    fn valid_major_numbers() -> &'static [usize];
}

sum_type::sum_type! {
    /// A union of all known operations.
    #[derive(Copy, Debug, Clone, PartialEq)]
    pub enum Op {
        LinearInterpolate,
        Dwell,
        Imperial,
        Metric,
        AbsoluteCoordinates,
        RelativeCoordinates,
    }
}

/// Convenience macro for deferring everything to each variant.
///

macro_rules! impl_op {
    ( $( $variant:ident),* ) => {
        impl Operation for Op {
            fn state_after(&self, seconds: Time, initial_state: State) -> State {
                match *self {
                    $(
                        Op::$variant(ref item) => item.state_after(seconds, initial_state),
                    )*
                }
            }

            fn duration(&self, initial_state: &State) -> Time {
                match *self {
                    $(
                        Op::$variant(ref item) => item.duration(initial_state),
                    )*
                }
            }
        }

        impl<'a> TryFrom<&'a Gcode> for Op {
            type Error = ConversionError;

            fn try_from(other: &'a Gcode) -> Result<Self, Self::Error> {
                let major = other.major_number();

                $(
                    if $variant::valid_major_numbers().contains(&major) {
                        return $variant::try_from(other).map(Into::into);
                    }
                )*

                Err(ConversionError::IncorrectMajorNumber {
                    found: other.major_number(),
                    expected: Op::valid_major_numbers(),
                })
            }
        }

        impl TryFrom<Gcode> for Op {
            type Error = ConversionError;

            fn try_from(other: Gcode) -> Result<Self, Self::Error> {
                Op::try_from(&other)
            }
        }
    }
}

impl_op!(
    LinearInterpolate,
    Dwell,
    Imperial,
    Metric,
    AbsoluteCoordinates,
    RelativeCoordinates
);

impl<'a> FromGcode<'a> for Op {
    fn valid_major_numbers() -> &'static [usize] {
        &[0, 1, 4, 20, 21, 90, 91]
    }
}

/// The reason a `TryFrom` conversion failed.
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
            Imperial::valid_major_numbers(),
            Metric::valid_major_numbers(),
            AbsoluteCoordinates::valid_major_numbers(),
            RelativeCoordinates::valid_major_numbers(),
        ];

        // make sure our should_be vector is correct
        assert_eq!(
            should_be.len(),
            count,
            "There should be items from {} variants",
            count
        );

        let got = Op::valid_major_numbers();

        let mut got: Vec<_> = got.into_iter().cloned().collect();
        let mut should_be: Vec<_> =
            should_be.into_iter().flatten().cloned().collect();

        got.sort();
        should_be.sort();
        assert_eq!(got, should_be);
    }
}
