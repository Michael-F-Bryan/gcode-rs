use super::{ConversionError, FromGcode, Operation};
use crate::operations::helpers;
use crate::TryFrom;
use gcode::Gcode;
use state::State;
use uom::si::f32::Time;
use uom::si::time::{millisecond, second};

/// Wait for a period of time.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dwell(pub Time);

impl Dwell {
    pub fn new(duration: Time) -> Dwell {
        Dwell(duration)
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
        debug_assert!(seconds <= self.0);
        initial_state
    }

    fn duration(&self, _initial_state: &State) -> Time {
        self.0
    }
}

impl TryFrom<Gcode> for Dwell {
    type Error = ConversionError;

    fn try_from(other: Gcode) -> Result<Self, Self::Error> {
        const VALIDATION_MSG: &str = "Dwell times must be positive";
        helpers::check_major_number::<Dwell>(&other)?;

        fn try_convert<U>(
            letter: char,
            gcode: &Gcode,
        ) -> Option<Result<Dwell, ConversionError>>
        where
            U: uom::si::time::Unit + uom::Conversion<f32, T = f32>,
        {
            let value = gcode.value_for(letter)?;

            if value >= 0.0 {
                let t = Time::new::<U>(value);
                Some(Ok(Dwell::new(t)))
            } else {
                Some(Err(ConversionError::InvalidArgument {
                    letter,
                    value,
                    message: VALIDATION_MSG,
                }))
            }
        }

        if let Some(h) = try_convert::<millisecond>('H', &other) {
            return h;
        } else if let Some(p) = try_convert::<millisecond>('P', &other) {
            return p;
        } else if let Some(s) = try_convert::<second>('S', &other) {
            return s;
        } else {
            Err(ConversionError::MissingArguments {
                expected: &['H', 'P', 'S'],
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
                    expected: &['H', 'P', 'S'],
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
            ("G4 P5", Ok(Dwell::from_milliseconds(5.0))),
            ("G4 S5", Ok(Dwell::from_seconds(5.0))),
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
