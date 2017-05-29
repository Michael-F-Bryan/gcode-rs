//! The high level interpreting of parsed Commands as their particular G and M
//! codes and applying strong typing to their arguments.

#![allow(missing_docs, dead_code)]

use low_level;

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
    unimplemented!()
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
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use low_level::Argument;

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
}
