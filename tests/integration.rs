//! Integration tests which try run the lexer and parser on valid gcode
//! programs and make sure it doesn't fail or panic.

extern crate gcode;

use gcode::Parser;


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

            let mut parser = Parser::new();

            for command in parser.parse(src.as_bytes()) {
                println!("{:?}", command);
            }
        }
    }
}

macro_rules! integration_tests {
    ($( $name:ident => $filename:expr),* ) => (
        $( integration_test!($name => $filename); )*
    )
}



// integration_tests!(program_1 => "data/program_1.gcode",
//                    program_2 => "data/program_2.gcode",
//                    program_3 => "data/program_3.gcode",
//                    guide => "data/guide.gcode");

// octocat => "data/PI_octcat.gcode",
// rust_logo => "data/PI_rustlogo.gcode"
