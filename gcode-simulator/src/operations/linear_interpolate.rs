use super::{ConversionError, FromGcode, Operation};
use crate::operations::helpers;
use crate::state::{AxisPositions, CoordinateMode, State};
use crate::TryFrom;
use gcode::Gcode;
#[allow(unused_imports)]
use libm::F32Ext;
use uom::si::f32::*;

/// Move directly from point A to B in a straight line.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct LinearInterpolate {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub feed_rate: Option<f32>,
}

impl LinearInterpolate {
    fn end_position(&self, start: &State) -> AxisPositions {
        match start.coordinate_mode {
            CoordinateMode::Absolute => {
                let mut pos = start.current_position;
                let LinearInterpolate {
                    x,
                    y,
                    feed_rate: _feed_rate,
                } = *self;
                if let Some(x) = x {
                    pos.x = start.to_length(x);
                }
                if let Some(y) = y {
                    pos.y = start.to_length(y);
                }
                pos
            }
            CoordinateMode::Relative => {
                start.current_position
                    + AxisPositions {
                        x: start.to_length(self.x.unwrap_or(0.0)),
                        y: start.to_length(self.y.unwrap_or(0.0)),
                    }
            }
        }
    }
}

impl Operation for LinearInterpolate {
    fn state_after(&self, duration: Time, initial_state: State) -> State {
        let end = self.end_position(&initial_state);
        let start = initial_state.current_position;

        let ratio = duration.value / self.duration(&initial_state).value;
        let new_pos = start + (end - start) * ratio;
        #[cfg(test)]
        println!("{:?} => {:?} = {:?}", start, end, new_pos);

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

        let end = self.end_position(initial_state);
        let start = initial_state.current_position;
        (end - start).length() / feed_rate
    }
}

impl TryFrom<Gcode> for LinearInterpolate {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        LinearInterpolate::try_from(&other)
    }
}

impl<'a> TryFrom<&'a Gcode> for LinearInterpolate {
    type Error = ConversionError;

    fn try_from(other: &'a Gcode) -> Result<Self, Self::Error> {
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

impl<'a> FromGcode<'a> for LinearInterpolate {
    fn valid_major_numbers() -> &'static [usize] {
        &[0, 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gcode;
    use uom::si::length::millimeter;

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

            let should_be = AxisPositions {
                x: Length::new::<millimeter>(should_be.0),
                y: Length::new::<millimeter>(should_be.1),
            };
            assert_relative_eq!(got, should_be);
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

            let should_be = AxisPositions {
                x: Length::new::<millimeter>(should_be.0),
                y: Length::new::<millimeter>(should_be.1),
            };
            assert_relative_eq!(got, should_be,);
        }
    }

    #[test]
    fn linear_interpolate_duration() {
        let initial_state = State::default()
            .with_x(50.0)
            .with_y(100.0)
            .with_coordinate_mode(CoordinateMode::Absolute);
        let feed_rate = 10.0;
        let input = LinearInterpolate {
            x: Some(10.0),
            y: Some(10.0),
            feed_rate: Some(feed_rate),
        };

        let dx = 10.0 - 50.0;
        let dy = 10.0 - 100.0;
        let distance = f32::hypot(dx, dy);

        let should_be = Length::new::<millimeter>(distance) / initial_state.to_speed(feed_rate);
        let got = input.duration(&initial_state);

        assert_relative_eq!(got.value, should_be.value, epsilon = 0.01);
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
