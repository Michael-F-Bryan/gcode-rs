//! The high level interpreting of parsed Commands as their particular G and M
//! codes and applying strong typing to their arguments.

#![allow(missing_docs, dead_code, unused_variables)]

use low_level::{self, Argument, ArgumentKind};
use errors::*;

/// Convert the loosely typed `low_level::Line` into its more strongly typed
/// representation.
///
/// Note that as a result of this process you tend to lose line information.
/// It's assumed that if you get this far in the pipeline you've already dealt
/// with errors.
pub fn type_check(line: low_level::Line) -> Result<Line> {
    match line {
        low_level::Line::ProgramNumber(n) => Ok(Line::ProgramNumber(n)),
        low_level::Line::Cmd(cmd) => convert_command(&cmd),
    }
}


fn convert_command(cmd: &low_level::Command) -> Result<Line> {
    match cmd.command() {
        (low_level::CommandType::M, n) => Ok(Line::M(convert_m(n, cmd.args())?)),
        (low_level::CommandType::G, n) => Ok(Line::G(convert_g(n, cmd.args())?)),
        (low_level::CommandType::T, n) => Ok(Line::T(n)),
    }
}

/// Convert a G code into its strongly-typed variant.
fn convert_g(number: u32, args: &[Argument]) -> Result<GCode> {
    let arg_reader = ArgumentReader::read(args);

    match number {
        0 => {
            if arg_reader.to.is_none() {
                return Err(Error::InvalidCommand("G00 must have at least one axis word specified"));
            }

            Ok(GCode::G00 {
                   to: arg_reader.to,
                   feed_rate: arg_reader.feed_rate,
               })
        }
        1 => {
            if arg_reader.to.is_none() {
                return Err(Error::InvalidCommand("G01 must have at least one axis word specified"));
            }

            Ok(GCode::G01 {
                   to: arg_reader.to,
                   feed_rate: arg_reader.feed_rate,
               })
        }
        other => panic!("G Code not yet supported: {}", other),
    }
}


fn convert_m(number: u32, args: &[Argument]) -> Result<MCode> {
    let arg_reader = ArgumentReader::read(args);

    match number {
        other => panic!("G Code not yet supported: {}", other),
    }
}



#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Line {
    /// A G code.
    G(GCode),
    /// A M code.
    M(MCode),
    /// A tool Change.
    T(u32),
    /// The program number.
    ProgramNumber(u32),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GCode {
    /// Rapid Linear Motion
    G00 { to: Point, feed_rate: Option<f32> },
    /// Linear Motion at Feed Rate
    G01 { to: Point, feed_rate: Option<f32> },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MCode {}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

impl Point {
    /// Check whether all the `Point`'s components are `None`.
    fn is_none(&self) -> bool {
        self.x.is_none() && self.y.is_none() && self.z.is_none()
    }

    fn set_x(&mut self, val: f32) {
        self.x = Some(val);
    }
    fn set_y(&mut self, val: f32) {
        self.y = Some(val);
    }
    fn set_z(&mut self, val: f32) {
        self.z = Some(val);
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct ArgumentReader {
    to: Point,
    feed_rate: Option<f32>,
}

impl ArgumentReader {
    fn read(arguments: &[Argument]) -> Self {
        let mut this = ArgumentReader::default();

        for arg in arguments {
            match arg.kind {
                ArgumentKind::X => this.to.set_x(arg.value),
                ArgumentKind::Y => this.to.set_y(arg.value),
                ArgumentKind::Z => this.to.set_z(arg.value),
                ArgumentKind::FeedRate => this.feed_rate = Some(arg.value),

                other => panic!(r#"Argument Kind "{:?}" isn't yet supported"#, other),
            }
        }

        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use low_level::{Argument, ArgumentKind};

    /// This creates a test which will try to convert the provided input into a
    /// GCode, then make sure we got back what we expect.
    macro_rules! g_code_test {
        ($name:ident, $input:expr => $output:expr) => {
            #[test]
            fn $name() {
                let input: (u32, &[Argument]) = $input;
                let should_be: GCode = $output;

                let got = convert_g(input.0, input.1).unwrap();
                assert_eq!(got, should_be);
            }
        }
    }

    g_code_test!(g_00, (0, &[Argument::new(ArgumentKind::Y, 3.1415)])
                 => GCode::G00 {
                            to: Point {y: Some(3.1415), ..Default::default()},
                            feed_rate: None
                        });

    g_code_test!(g_01, (1, &[
                            Argument::new(ArgumentKind::X, 1.23),
                            Argument::new(ArgumentKind::Y, 4.0),
                            Argument::new(ArgumentKind::Z, 2.71828),
                            Argument::new(ArgumentKind::FeedRate, 9000.0)])
                 => GCode::G01 {
                            to: Point {
                                x: Some(1.23),
                                y: Some(4.0),
                                z: Some(2.71828),
                            },
                            feed_rate: Some(9000.0),
                        });

    #[test]
    fn argument_reader_handles_coords() {
        let input = vec![Argument::new(ArgumentKind::X, 1.23),
                         Argument::new(ArgumentKind::Y, 3.1415),
                         Argument::new(ArgumentKind::Z, -2.1)];

        let should_be = ArgumentReader {
            to: Point {
                x: Some(1.23),
                y: Some(3.1415),
                z: Some(-2.1),
            },
            ..Default::default()
        };

        let got = ArgumentReader::read(&input);

        assert_eq!(got, should_be);
    }
}
