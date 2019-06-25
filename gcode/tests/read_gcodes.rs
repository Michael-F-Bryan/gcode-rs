#[macro_use]
extern crate pretty_assertions;
extern crate gcode;
use gcode::{Argument, Gcode, Mnemonic, Span};

#[test]
fn read_each_line_of_a_file() {
    let src = "O1000
        T1 M6
        (Linear / Feed - Absolute)
        G0 G90 G40 G21 G17 G94 G80
        G54 X-75 Y-75 S500 M3  (Position 6)
        G43 Z100 H1
        G01 Z5
        N42 G01 Z-20 F100";

    let got: Vec<_> = gcode::parse(src).collect();

    let should_be = vec![
        Gcode::new(Mnemonic::ProgramNumber, 1000.0).with_span(Span::new(0, 5, 0)),
        Gcode::new(Mnemonic::ToolChange, 1.0).with_span(Span::new(14, 17, 1)),
        Gcode::new(Mnemonic::Miscellaneous, 6.0).with_span(Span::new(17, 19, 1)),
        Gcode::new(Mnemonic::General, 0.0).with_span(Span::new(63, 66, 3)),
        Gcode::new(Mnemonic::General, 90.0).with_span(Span::new(66, 70, 3)),
        Gcode::new(Mnemonic::General, 40.0).with_span(Span::new(70, 74, 3)),
        Gcode::new(Mnemonic::General, 21.0).with_span(Span::new(74, 78, 3)),
        Gcode::new(Mnemonic::General, 17.0).with_span(Span::new(78, 82, 3)),
        Gcode::new(Mnemonic::General, 94.0).with_span(Span::new(82, 86, 3)),
        Gcode::new(Mnemonic::General, 80.0).with_span(Span::new(86, 89, 3)),
        Gcode::new(Mnemonic::General, 54.0)
            .with_span(Span::new(98, 102, 4))
            .with_argument(Argument::new('X', -75.0).with_span(Span::new(102, 107, 4)))
            .with_argument(Argument::new('Y', -75.0).with_span(Span::new(107, 112, 4)))
            .with_argument(Argument::new('S', 500.0).with_span(Span::new(112, 117, 4))),
        Gcode::new(Mnemonic::Miscellaneous, 3.0).with_span(Span::new(117, 121, 4)),
        Gcode::new(Mnemonic::General, 43.0)
            .with_span(Span::new(142, 146, 5))
            .with_argument(Argument::new('Z', 100.0).with_span(Span::new(146, 151, 5)))
            .with_argument(Argument::new('H', 1.0).with_span(Span::new(151, 153, 5))),
        Gcode::new(Mnemonic::General, 1.0)
            .with_span(Span::new(162, 166, 6))
            .with_argument(Argument::new('Z', 5.0).with_span(Span::new(166, 168, 6))),
        Gcode::new(Mnemonic::General, 1.0)
            .with_span(Span::new(181, 185, 7))
            .with_argument(Argument::new('Z', -20.0).with_span(Span::new(185, 190, 7)))
            .with_argument(Argument::new('F', 100.0).with_span(Span::new(190, 194, 7)))
            .with_line_number(42),
    ];

    assert_eq!(got, should_be);
}

macro_rules! parse_fixture {
    ($test_name:ident, $filename:expr => $num_codes:expr) => {
        #[test]
        fn $test_name() {
            let src = include_str!(concat!("data/", $filename));
            let num_gcodes = gcode::parse(src).count();

            assert_eq!(num_gcodes, $num_codes);
        }
    };
}

parse_fixture!(parse_program_1, "program_1.gcode" => 24);
parse_fixture!(parse_program_2, "program_2.gcode" => 14);
// Doesn't work because it uses "S" as a command
//parse_fixture!(parse_program_3, "program_3.gcode" => 410);

// These guys take forever to parse...
//parse_fixture!(parse_octocat, "PI_octcat.gcode" => 145362);
//parse_fixture!(parse_rust_logo, "PI_rustlogo.gcode" => 195701);
