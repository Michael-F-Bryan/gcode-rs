use super::{ConversionError, FromGcode};
use gcode::Gcode;

/// Create an operation which takes no arguments, contains no data, and only
/// does one thing.
macro_rules! singleton_cmd {
    ($( #[$attr:meta] )* $name:ident, $number:expr, |$state:ident| $new_state:expr) => {

        $(
            #[$attr]
        )*
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct $name;

        impl $crate::operations::Operation for $name {
            fn state_after(&self, _seconds: uom::si::f32::Time, $state: $crate::State) -> $crate::State {
                $new_state
            }

            fn duration(&self, _initial_state: &$crate::State) -> uom::si::f32::Time {
                use uom::num::Zero;
                uom::si::f32::Time::zero()
            }
        }

        impl $crate::operations::FromGcode for $name {
            fn valid_major_numbers() -> &'static [usize] {
                &[$number]
            }
        }

        impl $crate::operations::TryFrom<gcode::Gcode> for $name {
            type Error = $crate::operations::ConversionError;

            fn try_from(other: gcode::Gcode) -> Result<Self, Self::Error> {
                $crate::operations::helpers::check_major_number::<$name>(&other)?;

                Ok($name)
            }
        }
    };
}

pub(crate) fn check_major_number<T: FromGcode>(
    gcode: &Gcode,
) -> Result<(), ConversionError> {
    let valid_numbers = T::valid_major_numbers();
    let major = gcode.major_number();

    if valid_numbers.contains(&major) {
        Ok(())
    } else {
        Err(ConversionError::IncorrectMajorNumber {
            found: major,
            expected: valid_numbers,
        })
    }
}
