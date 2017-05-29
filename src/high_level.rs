#![allow(missing_docs, dead_code)]

use parser;

#[derive(Debug)]
pub struct Parser<I>
    where I: Iterator<Item = parser::Line>
{
    src: I,
}


impl<I> Parser<I>
    where I: Iterator<Item = parser::Line>
{
    pub fn new(tokens: I) -> Parser<I> {
        Parser { src: tokens }
    }

    pub fn next_command(&mut self) -> Option<Line> {
        if let Some(next) = self.src.next() {
            match next {
                parser::Line::ProgramNumber(n) => Some(Line::ProgramNumber(n)),
                parser::Line::Cmd(cmd) => Some(self.parse_command(cmd)),
            }
        } else {
            None
        }
    }

    fn parse_command(&self, cmd: parser::Command) -> Line {
        match cmd.command() {
            (parser::CommandType::M, _) => self.parse_m(cmd),
            (parser::CommandType::G, _) => self.parse_g(cmd),
            (parser::CommandType::T, _) => self.parse_t(cmd),
        }
    }
    fn parse_g(&self, cmd: parser::Command) -> Line {
        unimplemented!()
    }

    fn parse_t(&self, cmd: parser::Command) -> Line {
        unimplemented!()
    }

    fn parse_m(&self, cmd: parser::Command) -> Line {
        unimplemented!()
    }
}

impl<I> Iterator for Parser<I>
    where I: Iterator<Item = parser::Line>
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
