//! Integration tests which try run the lexer on valid gcode programs and make
//! sure it doesn't fail or panic.

extern crate gcode;

use gcode::Tokenizer;
use gcode::{Result, Error};

const PROGRAM_1: &'static str = include_str!("data/program_1.gcode");
const PROGRAM_2: &'static str = include_str!("data/program_2.gcode");
const PROGRAMS: [&'static str; 2] = [PROGRAM_1, PROGRAM_2];

#[test]
fn lex_all_programs() {
    for program in &PROGRAMS {
        let mut tokenizer = Tokenizer::new(program.chars());
        let tokens: Result<Vec<_>> = tokenizer.collect();

        tokens.unwrap();
    }
}
