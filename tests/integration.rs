//! Integration tests which try run the lexer and parser on valid gcode
//! programs and make sure it doesn't fail or panic.

extern crate gcode;

use gcode::lexer::Tokenizer;
use gcode::BasicParser;

macro_rules! lex_file {
    ($name:ident => $filename:expr) => {
        #[test]
        fn $name() {
            println!("Lexing Program: {}", $filename);

            let src = include_str!($filename);
            for token in Tokenizer::new(src.chars()) {
                println!("{:?}", token.unwrap());
            }
        }
    }
}

macro_rules! parse_file {
    ($name:ident => $filename:expr) => {
        #[test]
        fn $name() {
            println!("Parsing Program: {}", $filename);
            println!();

            let src = include_str!($filename);
            let tokens = Tokenizer::new(src.chars());

            for command in BasicParser::new(tokens.filter_map(|t| t.ok())) {
                println!("{:?}", command.unwrap());
            }
        }
    }
}

macro_rules! lex_files {
    ($( $name:ident => $filename:expr),* ) => (
        $( lex_file!($name => $filename); )*
    )
}
macro_rules! parse_files {
    ($( $name:ident => $filename:expr),* ) => (
        $( parse_file!($name => $filename); )*
    )
}



lex_files!(lex_program_1 => "data/program_1.gcode",
           lex_program_2 => "data/program_2.gcode",
           lex_program_3 => "data/program_3.gcode");

parse_files!(parse_program_1 => "data/program_1.gcode",
           parse_program_2 => "data/program_2.gcode",
           parse_program_3 => "data/program_3.gcode");
