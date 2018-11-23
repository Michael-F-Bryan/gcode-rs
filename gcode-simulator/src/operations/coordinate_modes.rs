use super::{ConversionError, FromGcode, Operation};
use crate::state::{CoordinateMode, State};
use crate::TryFrom;
use gcode::Gcode;
use uom::num::Zero;
use uom::si::f32::*;

/// Tell the machine to use absolute coordinates.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AbsoluteCoordinates;

impl Operation for AbsoluteCoordinates {
    fn state_after(&self, _seconds: Time, initial_state: State) -> State {
        initial_state.with_coordinate_mode(CoordinateMode::Absolute)
    }

    fn duration(&self, _initial_state: &State) -> Time {
        Time::zero()
    }
}

impl FromGcode for AbsoluteCoordinates {
    fn valid_major_numbers() -> &'static [usize] {
        &[90]
    }
}

impl TryFrom<Gcode> for AbsoluteCoordinates {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        super::check_major_number::<AbsoluteCoordinates>(&other)?;

        Ok(AbsoluteCoordinates)
    }
}

/// Tell the machine to use relative coordinates.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RelativeCoordinates;

impl Operation for RelativeCoordinates {
    fn state_after(&self, _seconds: Time, initial_state: State) -> State {
        initial_state.with_coordinate_mode(CoordinateMode::Relative)
    }

    fn duration(&self, _initial_state: &State) -> Time {
        Time::zero()
    }
}

impl FromGcode for RelativeCoordinates {
    fn valid_major_numbers() -> &'static [usize] {
        &[91]
    }
}

impl TryFrom<Gcode> for RelativeCoordinates {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        super::check_major_number::<RelativeCoordinates>(&other)?;

        Ok(RelativeCoordinates)
    }
}
