use super::{ConversionError, FromGcode, Operation};
use crate::operations::helpers;
use crate::TryFrom;
use gcode::Gcode;
#[allow(unused_imports)]
use libm::F32Ext;
use state::{AxisPositions, CoordinateMode, State};
use uom::si::f32::*;
use uom::si::Quantity;

/// Move directly from point A to B in a straight line.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct LinearInterpolate {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub feed_rate: Option<f32>,
}

impl LinearInterpolate {
    fn end_position(&self, start: &State) -> (Length, Length) {
        match start.coordinate_mode {
            CoordinateMode::Absolute => {
                let x = self.x.map(|x| start.to_length(x));
                let y = self.y.map(|y| start.to_length(y));
                (
                    x.unwrap_or(start.current_position.x),
                    y.unwrap_or(start.current_position.y),
                )
            }
            CoordinateMode::Relative => (
                start.current_position.x
                    + start.to_length(self.x.unwrap_or(0.0)),
                start.current_position.y
                    + start.to_length(self.y.unwrap_or(0.0)),
            ),
        }
    }

    pub fn lerp(start: Length, end: Length, proportion: f32) -> Length {
        debug_assert!(0.0 <= proportion && proportion <= 1.0);
        let diff = end - start;
        start + diff * proportion
    }
}

impl Operation for LinearInterpolate {
    fn state_after(&self, duration: Time, initial_state: State) -> State {
        let (end_x, end_y) = self.end_position(&initial_state);
        let AxisPositions {
            x: start_x,
            y: start_y,
        } = initial_state.current_position;

        let ratio = duration.value / self.duration(&initial_state).value;
        let new_pos = AxisPositions {
            x: LinearInterpolate::lerp(start_x, end_x, ratio),
            y: LinearInterpolate::lerp(start_y, end_y, ratio),
        };

        let feed_rate = self
            .feed_rate
            .map(|f| initial_state.to_speed(f))
            .unwrap_or(initial_state.feed_rate);

        State {
            current_position: new_pos,
            feed_rate,
            ..initial_state
        }
    }

    fn duration(&self, initial_state: &State) -> Time {
        let feed_rate = self
            .feed_rate
            .map(|f| initial_state.to_speed(f))
            .unwrap_or(initial_state.feed_rate);
        let feed_rate_mps = feed_rate / 60.0;
        let (end_x, end_y) = self.end_position(initial_state);

        let AxisPositions { x, y } = initial_state.current_position;
        let dx = end_x - x;
        let dy = end_y - y;
        let distance = hypot(dx, dy);
        distance / feed_rate_mps
    }
}

fn hypot<D, U>(
    left: Quantity<D, U, f32>,
    right: Quantity<D, U, f32>,
) -> Quantity<D, U, f32>
where
    D: uom::si::Dimension + ?Sized,
    U: uom::si::Units<f32> + ?Sized,
{
    let value = f32::hypot(left.value, right.value);
    Quantity { value, ..left }
}

impl TryFrom<Gcode> for LinearInterpolate {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        helpers::check_major_number::<LinearInterpolate>(&other)?;

        let x = other.value_for('X');
        let y = other.value_for('Y');

        if x.is_none() && y.is_none() {
            return Err(ConversionError::MissingArguments {
                expected: &['X', 'Y'],
            });
        }

        let feed_rate = other.value_for('F');

        if let Some(f) = feed_rate {
            if f <= 0.0 {
                return Err(ConversionError::InvalidArgument {
                    letter: 'F',
                    value: f,
                    message: "Feed rate must always be a positive number",
                });
            }
        }

        Ok(LinearInterpolate { x, y, feed_rate })
    }
}

impl FromGcode for LinearInterpolate {
    fn valid_major_numbers() -> &'static [usize] {
        &[0, 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gcode;

    #[test]
    fn parse_a_linear_interpolate() {
        let inputs = vec![
            (
                "G00 X50.0",
                Ok(LinearInterpolate {
                    x: Some(50.0),
                    y: None,
                    feed_rate: None,
                }),
            ),
            (
                "G90",
                Err(ConversionError::IncorrectMajorNumber {
                    found: 90,
                    expected: &[0, 1],
                }),
            ),
            (
                "G01 X5 Y-20.5 F10000",
                Ok(LinearInterpolate {
                    x: Some(5.0),
                    y: Some(-20.5),
                    feed_rate: Some(10000.0),
                }),
            ),
            (
                "G01 Y-20.5",
                Ok(LinearInterpolate {
                    x: None,
                    y: Some(-20.5),
                    feed_rate: None,
                }),
            ),
            (
                "G01 F10000",
                Err(ConversionError::MissingArguments {
                    expected: &['X', 'Y'],
                }),
            ),
            (
                "G01 X5 Y-20.5 F-10000",
                Err(ConversionError::InvalidArgument {
                    letter: 'F',
                    value: -10000.0,
                    message: "Feed rate must always be a positive number",
                }),
            ),
        ];

        for (src, should_be) in inputs {
            let raw = gcode::parse(src).next().unwrap();
            let got = LinearInterpolate::try_from(raw);

            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn linear_interpolate_end_position_absolute() {
        let initial_state = State::default()
            .with_x(50.0)
            .with_y(100.0)
            .with_coordinate_mode(CoordinateMode::Absolute);
        let inputs = vec![
            (
                LinearInterpolate {
                    x: Some(10.0),
                    y: Some(10.0),
                    ..Default::default()
                },
                (10.0, 10.0),
            ),
            (
                LinearInterpolate {
                    x: Some(10.0),
                    y: None,
                    ..Default::default()
                },
                (10.0, 100.0),
            ),
            (
                LinearInterpolate {
                    x: None,
                    y: Some(123.0),
                    ..Default::default()
                },
                (50.0, 123.0),
            ),
        ];

        for (src, should_be) in inputs {
            let got = src.end_position(&initial_state);
            assert_relative_eq!(
                got.0.value,
                initial_state.to_length(should_be.0).value
            );
            assert_relative_eq!(
                got.1.value,
                initial_state.to_length(should_be.1).value
            );
        }
    }

    #[test]
    fn linear_interpolate_end_position_relative() {
        let initial_state = State::default()
            .with_x(50.0)
            .with_y(100.0)
            .with_coordinate_mode(CoordinateMode::Relative);
        let inputs = vec![
            (
                LinearInterpolate {
                    x: Some(10.0),
                    y: Some(10.0),
                    ..Default::default()
                },
                (60.0, 110.0),
            ),
            (
                LinearInterpolate {
                    x: Some(10.0),
                    y: None,
                    ..Default::default()
                },
                (60.0, 100.0),
            ),
            (
                LinearInterpolate {
                    x: None,
                    y: Some(123.0),
                    ..Default::default()
                },
                (50.0, 223.0),
            ),
        ];

        for (src, should_be) in inputs {
            let got = src.end_position(&initial_state);
            assert_relative_eq!(
                got.0.value,
                initial_state.to_length(should_be.0).value
            );
            assert_relative_eq!(
                got.1.value,
                initial_state.to_length(should_be.1).value
            );
        }
    }

    #[test]
    fn linear_interpolate_duration() {
        let initial_state = State::default()
            .with_x(50.0)
            .with_y(100.0)
            .with_coordinate_mode(CoordinateMode::Absolute);
        let input = LinearInterpolate {
            x: Some(10.0),
            y: Some(10.0),
            feed_rate: Some(10.0),
        };

        let dx = 10.0 - 50.0;
        let dy = 10.0 - 100.0;
        let distance = f32::hypot(dx, dy);

        let should_be = distance * 60.0 / 10.0;
        let got = input.duration(&initial_state);

        assert_relative_eq!(got.value, should_be);
    }

    #[test]
    fn linear_interpolate_state_after() {
        let initial_state = State::default()
            .with_x(50.0)
            .with_y(100.0)
            .with_coordinate_mode(CoordinateMode::Absolute);
        let input = LinearInterpolate {
            x: Some(10.0),
            y: None,
            feed_rate: Some(10.0),
        };

        let t = input.duration(&initial_state) / 2.0;
        let should_be = initial_state
            .clone()
            .with_x((50.0 + 10.0) / 2.0)
            .with_y(100.0)
            .with_feed_rate(10.0);

        let got = input.state_after(t, initial_state);

        assert_eq!(got, should_be);
    }
}
