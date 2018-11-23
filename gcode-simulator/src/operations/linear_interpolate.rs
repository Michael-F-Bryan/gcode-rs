use super::{ConversionError, FromGcode, Operation};
use crate::TryFrom;
use gcode::Gcode;
#[allow(unused_imports)]
use libm::F32Ext;
use state::{CoordinateMode, State};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct LinearInterpolate {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub feed_rate: Option<f32>,
}

impl LinearInterpolate {
    fn end_position(&self, start: &State) -> (f32, f32) {
        match start.coordinate_mode {
            CoordinateMode::Absolute => {
                (self.x.unwrap_or(start.x), self.y.unwrap_or(start.y))
            }
            CoordinateMode::Relative => (
                start.x + self.x.unwrap_or(0.0),
                start.y + self.y.unwrap_or(0.0),
            ),
        }
    }
}

impl Operation for LinearInterpolate {
    fn state_after(&self, duration: f32, initial_state: State) -> State {
        let feed_rate = self.feed_rate.unwrap_or(initial_state.feed_rate);
        let (end_x, end_y) = self.end_position(&initial_state);

        let dx = end_x - initial_state.x;
        let dy = end_y - initial_state.y;
        let total_duration = self.duration(&initial_state);
        let ratio = duration / total_duration;
        let x = initial_state.x + dx * ratio;
        let y = initial_state.y + dy * ratio;

        State {
            x,
            y,
            feed_rate,
            ..initial_state
        }
    }

    fn duration(&self, initial_state: &State) -> f32 {
        let feed_rate = self.feed_rate.unwrap_or(initial_state.feed_rate);
        let feed_rate_mps = feed_rate / 60.0;
        let (end_x, end_y) = self.end_position(initial_state);

        let dx = end_x - initial_state.x;
        let dy = end_y - initial_state.y;
        let distance = f32::hypot(dx, dy);
        distance / feed_rate_mps
    }
}

impl TryFrom<Gcode> for LinearInterpolate {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        let maj = other.major_number();
        let valid_numbers = LinearInterpolate::valid_major_numbers();

        if !valid_numbers.contains(&maj) {
            return Err(ConversionError::IncorrectMajorNumber {
                found: maj,
                expected: valid_numbers,
            });
        }

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
        let initial_state = State {
            x: 50.0,
            y: 100.0,
            coordinate_mode: CoordinateMode::Absolute,
            ..Default::default()
        };
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
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn linear_interpolate_end_position_relative() {
        let initial_state = State {
            x: 50.0,
            y: 100.0,
            coordinate_mode: CoordinateMode::Relative,
            ..Default::default()
        };
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
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn linear_interpolate_duration() {
        let initial_state = State {
            x: 50.0,
            y: 100.0,
            coordinate_mode: CoordinateMode::Absolute,
            ..Default::default()
        };
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

        assert_relative_eq!(got, should_be);
    }

    #[test]
    fn linear_interpolate_state_after() {
        let initial_state = State {
            x: 50.0,
            y: 100.0,
            coordinate_mode: CoordinateMode::Absolute,
            ..Default::default()
        };
        let input = LinearInterpolate {
            x: Some(10.0),
            y: None,
            feed_rate: Some(10.0),
        };

        let t = input.duration(&initial_state) / 2.0;
        let should_be = State {
            x: (50.0 + 10.0) / 2.0,
            y: 100.0,
            feed_rate: 10.0,
            ..initial_state
        };

        let got = input.state_after(t, initial_state);

        assert_eq!(got, should_be);
    }

    #[test]
    fn feed_rate_is_units_per_min() {
        let initial_state = State::default();
        let input = LinearInterpolate {
            x: Some(100.0),
            y: None,
            feed_rate: Some(100.0),
        };

        let duration = input.duration(&initial_state);
        assert_eq!(duration, 60.0);
    }
}
