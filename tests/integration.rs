//! Integration tests which try run the lexer and parser on valid gcode
//! programs and make sure it doesn't fail or panic.

extern crate gcode;

use gcode::{Tokenizer, Parser};

/// Create an integration test which will take the gcodes from the specified
/// file, then run the lexer and low level parser in stages, making sure that
/// each stage had no errors.
macro_rules! integration_test {
    ($name:ident => $filename:expr) => {
        #[test]
        fn $name() {
            println!("Testing Program: {}", $filename);
            println!();

            let src = include_str!($filename);

            println!();
            println!("Tokenizing");
            println!("==========");
            println!();

            let lexer = Tokenizer::new(src.chars());
            let tokens = lexer.inspect(|t| println!("{:?}", t))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            println!();
            println!("Low Level Parsing");
            println!("=================");
            println!();

            let low_level_parser = Parser::new(tokens.into_iter());
            let lines = low_level_parser
                .inspect(|c| println!("{:?}", c))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();


            for line in lines {
                println!("{:?}", line);
            }
        }
    }
}

macro_rules! integration_tests {
    ($( $name:ident => $filename:expr),* ) => (
        $( integration_test!($name => $filename); )*
    )
}



integration_tests!(program_1 => "data/program_1.gcode",
                   program_2 => "data/program_2.gcode",
                   program_3 => "data/program_3.gcode",
                   guide => "data/guide.gcode");

// octocat => "data/PI_octcat.gcode",
// rust_logo => "data/PI_rustlogo.gcode"
