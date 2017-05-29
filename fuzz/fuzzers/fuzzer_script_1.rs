#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate gcode;

use gcode::{Tokenizer, BasicParser, type_check};

fuzz_target!(|data: &[u8]| {
    let src = String::from_utf8_lossy(data);

    let lexer = Tokenizer::new(src.chars());
    let tokens = lexer.filter_map(|t| t.ok());

    BasicParser::new(tokens)
        .filter_map(|l| l.ok())
        // .map(type_check)
        // .filter_map(|l| l.ok())
        .collect::<Vec<_>>();
});
