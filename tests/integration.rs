//! Integration tests which try run the lexer and parser on valid gcode
//! programs and make sure it doesn't fail or panic.

extern crate gcode;

use gcode::lexer::Tokenizer;
use gcode::parser::Parser;
use gcode::Result;

const PROGRAMS: [&'static str; 3] = [include_str!("data/program_1.gcode"),
                                     include_str!("data/program_2.gcode"),
                                     include_str!("data/program_3.gcode")];

#[test]
fn parse_all_programs() {
    for (i, program) in PROGRAMS.iter().enumerate() {
        println!("Program {}", i + 1);

        let tokenizer = Tokenizer::new(program.chars());
        let tokens = tokenizer.filter_map(|t| t.ok());

        let parsed_commands: Result<Vec<_>> = Parser::new(tokens).collect();
        parsed_commands.unwrap();
    }
}

#[test]
fn lex_all_programs() {
    for (i, program) in PROGRAMS.iter().enumerate() {
        println!("Program {}", i + 1);
        let tokenizer = Tokenizer::new(program.chars());
        let tokens: Result<Vec<_>> = tokenizer.collect();

        tokens.unwrap();
    }
}
