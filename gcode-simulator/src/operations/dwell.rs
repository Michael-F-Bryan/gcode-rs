use super::{ConversionError, Operation};
use crate::TryFrom;
use gcode::Gcode;
use state::State;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dwell {
    /// The number of seconds to wait for.
    pub duration: f32,
}

impl Dwell {
    pub fn new(duration: f32) -> Dwell {
        Dwell { duration }
    }
}

impl Operation for Dwell {
    fn state_after(&self, seconds: f32, initial_state: State) -> State {
        debug_assert!(seconds <= self.duration);
        initial_state
    }

    fn duration(&self, _initial_state: &State) -> f32 {
        self.duration
    }
}

impl TryFrom<Gcode> for Dwell {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        const VALIDATION_MSG: &str = "Dwell times must be positive";

        if other.major_number() != 4 {
            return Err(ConversionError::IncorrectMajorNumber {
                found: other.major_number(),
                expected: &[4],
            });
        }

        if let Some(h) = other.value_for('H') {
            if h >= 0.0 {
                Ok(Dwell::new(h / 1000.0))
            } else {
                Err(ConversionError::InvalidArgument {
                    letter: 'H',
                    value: h,
                    message: VALIDATION_MSG,
                })
            }
        } else if let Some(p) = other.value_for('P') {
            if p >= 0.0 {
                Ok(Dwell::new(p))
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
            ("G4 H5", Ok(Dwell::new(5e-3))),
            ("G4 P5", Ok(Dwell::new(5.0))),
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
