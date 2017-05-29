extern crate gcode;

use gcode::lexer::Tokenizer;
use gcode::BasicParser;


fn main() {
    let src = include_str!("../tests/data/PI_octcat.gcode");

    let lexer = Tokenizer::new(src.chars());

    // We want an iterator which only gives us valid tokens, skipping invalid
    // ones
    let tokens = lexer.filter_map(|t| t.ok());

    let parser = BasicParser::new(tokens);

    for line in parser {
        println!("{}", line.unwrap());
    }
}
