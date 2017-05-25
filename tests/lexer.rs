//! Integration tests which try run the lexer on valid gcode programs and make
//! sure it doesn't fail or panic.

extern crate gcode;

use gcode::lexer::Tokenizer;
use gcode::Result;

const PROGRAMS: [&'static str; 3] = [include_str!("data/program_1.gcode"),
                                     include_str!("data/program_2.gcode"),
                                     include_str!("data/program_3.gcode")];

#[test]
fn lex_all_programs() {
    for program in &PROGRAMS {
        let tokenizer = Tokenizer::new(program.chars());
        let tokens: Result<Vec<_>> = tokenizer.collect();

        tokens.unwrap();
    }
}
