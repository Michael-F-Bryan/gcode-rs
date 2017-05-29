//! The high level interpreting of parsed Commands as their particular G and M
//! codes and applying strong typing to their arguments.

#![allow(missing_docs, dead_code)]

use low_level;

#[derive(Debug)]
pub struct Parser<I>
    where I: Iterator<Item = low_level::Line>
{
    src: I,
}


impl<I> Parser<I>
    where I: Iterator<Item = low_level::Line>
{
    pub fn new(tokens: I) -> Parser<I> {
        Parser { src: tokens }
    }

    pub fn next_command(&mut self) -> Option<Line> {
        if let Some(next) = self.src.next() {
            match next {
                low_level::Line::ProgramNumber(n) => Some(Line::ProgramNumber(n)),
                low_level::Line::Cmd(cmd) => Some(self.parse_command(cmd)),
            }
        } else {
            None
        }
    }

    fn parse_command(&self, cmd: low_level::Command) -> Line {
        match cmd.command() {
            (low_level::CommandType::M, _) => self.parse_m(cmd),
            (low_level::CommandType::G, _) => self.parse_g(cmd),
            (low_level::CommandType::T, _) => self.parse_t(cmd),
        }
    }
    fn parse_g(&self, cmd: low_level::Command) -> Line {
        unimplemented!()
    }

    fn parse_t(&self, cmd: low_level::Command) -> Line {
        unimplemented!()
    }

    fn parse_m(&self, cmd: low_level::Command) -> Line {
        unimplemented!()
    }
}

impl<I> Iterator for Parser<I>
    where I: Iterator<Item = low_level::Line>
{
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_command()
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    G(GCode),
    M(MCode),
    T(u32),
    ProgramNumber(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GCode {}

#[derive(Clone, Debug, PartialEq)]
pub enum MCode {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}
