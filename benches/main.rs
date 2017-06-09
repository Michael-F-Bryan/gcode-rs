#![feature(test)]
#![allow(deprecated)]
extern crate test;
extern crate gcode;

use gcode::{Tokenizer, BasicParser, Parser, Result};
// use gcode::type_check;


const SRC: &'static str = include_str!("../tests/data/program_3.gcode");

#[bench]
fn lex_program_3(b: &mut test::Bencher) {
    b.iter(|| {
               let lexer = Tokenizer::new(SRC.chars());
               lexer.collect::<Result<Vec<_>>>()
           })
}

#[bench]
fn bench_parse_program_3(b: &mut test::Bencher) {
    b.iter(|| {
               let lexer = Tokenizer::new(SRC.chars());
               let tokens = lexer.filter_map(|t| t.ok());
               let parser = BasicParser::new(tokens);
               parser.collect::<Result<Vec<_>>>().unwrap()
           })
}


#[bench]
fn new_parser(b: &mut test::Bencher) {
    b.iter(|| {
               let lexer = Tokenizer::new(SRC.chars());
               let tokens = lexer.filter_map(|t| t.ok());
               Parser::new(tokens).map(|l| l.unwrap()).collect::<Vec<_>>()
           })
}
