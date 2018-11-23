use super::{ConversionError, FromGcode, Operation};
use crate::TryFrom;
use gcode::Gcode;
use state::State;
use uom::si::f32::Time;
use uom::si::time::{millisecond, second};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dwell {
    /// The number of seconds to wait for.
    pub duration: Time,
}

impl Dwell {
    pub fn new(duration: Time) -> Dwell {
        Dwell { duration }
    }

    pub fn from_seconds(duration: f32) -> Dwell {
        Dwell::new(Time::new::<second>(duration))
    }

    pub fn from_milliseconds(duration: f32) -> Dwell {
        Dwell::new(Time::new::<millisecond>(duration))
    }
}

impl Operation for Dwell {
    fn state_after(&self, seconds: Time, initial_state: State) -> State {
        debug_assert!(seconds <= self.duration);
        initial_state
    }

    fn duration(&self, _initial_state: &State) -> Time {
        self.duration
    }
}

impl TryFrom<Gcode> for Dwell {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        const VALIDATION_MSG: &str = "Dwell times must be positive";
        let valid_numbers = Dwell::valid_major_numbers();

        if !valid_numbers.contains(&other.major_number()) {
            return Err(ConversionError::IncorrectMajorNumber {
                found: other.major_number(),
                expected: valid_numbers,
            });
        }

        if let Some(h) = other.value_for('H') {
            if h >= 0.0 {
                Ok(Dwell::new(Time::new::<millisecond>(h)))
            } else {
                Err(ConversionError::InvalidArgument {
                    letter: 'H',
                    value: h,
                    message: VALIDATION_MSG,
                })
            }
        } else if let Some(p) = other.value_for('P') {
            if p >= 0.0 {
                Ok(Dwell::new(Time::new::<second>(p)))
            } else {
                Err(ConversionError::InvalidArgument {
                    letter: 'P',
                    value: p,
                    message: VALIDATION_MSG,
                })
            }
        } else {
            Err(ConversionError::MissingArguments {
                expected: &['H', 'P'],
            })
        }
    }
}

impl FromGcode for Dwell {
    fn valid_major_numbers() -> &'static [usize] {
        &[4]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gcode;

    #[test]
    fn parse_a_dwell() {
        let inputs = vec![
            (
                "G4",
                Err(ConversionError::MissingArguments {
                    expected: &['H', 'P'],
                }),
            ),
            (
                "G00",
                Err(ConversionError::IncorrectMajorNumber {
                    found: 0,
                    expected: &[4],
                }),
            ),
            ("G4 H5", Ok(Dwell::from_milliseconds(5.0))),
            ("G4 P5", Ok(Dwell::from_seconds(5.0))),
            (
                "G4 P-50",
                Err(ConversionError::InvalidArgument {
                    letter: 'P',
                    value: -50.0,
                    message: "Dwell times must be positive",
                }),
            ),
        ];

        for (src, should_be) in inputs {
            let raw = gcode::parse(src).next().unwrap();
            let got = Dwell::try_from(raw);

            assert_eq!(got, should_be);
        }
    }
}
