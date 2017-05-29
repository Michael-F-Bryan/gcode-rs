//! The high level interpreting of parsed Commands as their particular G and M
//! codes and applying strong typing to their arguments.

#![allow(missing_docs, dead_code)]

use low_level::{self, Argument, ArgumentKind};

pub fn type_check(line: low_level::Line) -> Line {
    match line {
        low_level::Line::ProgramNumber(n) => Line::ProgramNumber(n),
        low_level::Line::Cmd(cmd) => convert_command(cmd),
    }
}


fn convert_command(cmd: low_level::Command) -> Line {
    match cmd.command() {
        (low_level::CommandType::M, n) => Line::M(convert_m(n, cmd.args())),
        (low_level::CommandType::G, n) => Line::G(convert_g(n, cmd.args())),
        (low_level::CommandType::T, n) => Line::T(n),
    }
}

fn convert_g(number: u32, args: &[low_level::Argument]) -> GCode {
    match number {
        0 => {
            let args = ArgumentReader::read(args);
            GCode::G00 {
                to: args.to,
                feed_rate: args.feed_rate,
            }
        }
        _ => unimplemented!(),
    }
}


fn convert_m(number: u32, args: &[low_level::Argument]) -> MCode {
    unimplemented!()
}



#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    G(GCode),
    M(MCode),
    T(u32),
    ProgramNumber(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GCode {
    G00 { to: Point, feed_rate: Option<f32> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MCode {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

impl Point {
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

#[derive(Clone, Debug, Default, PartialEq)]
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
                _ => unimplemented!(),
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

                let got = convert_g(input.0, input.1);
                assert_eq!(got, should_be);
            }
        }
    }

    g_code_test!(g_00, (0, &[]) => GCode::G00 { to: Point::default(), feed_rate: None });

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
