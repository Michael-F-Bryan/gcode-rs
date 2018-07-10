extern crate arrayvec;
extern crate gcode;

use arrayvec::ArrayVec;
use gcode::{Gcode, Mnemonic, Span, Word};

#[test]
fn read_each_line_of_a_file() {
    let src = "O1000
        T1 M6
        (Linear / Feed - Absolute)
        G0 G90 G40 G21 G17 G94 G80
        G54 X-75 Y-75 S500 M3  (Position 6)
        G43 Z100 H1
        G01 Z5
        G01 Z-20 F100";

    let got: Vec<_> = gcode::parse(src).collect();

    let should_be = vec![
        Gcode::new(Mnemonic::ProgramNumber, 1000.0, Span::new(0, 5, 0)),
        Gcode::new(Mnemonic::ToolChange, 1.0, Span::new(14, 16, 1)),
        Gcode::new(Mnemonic::MachineRoutine, 6.0, Span::new(17, 19, 1)),
        Gcode::new(Mnemonic::General, 0.0, Span::new(63, 65, 3)),
        Gcode::new(Mnemonic::General, 90.0, Span::new(66, 69, 3)),
        Gcode::new(Mnemonic::General, 40.0, Span::new(70, 73, 3)),
        Gcode::new(Mnemonic::General, 21.0, Span::new(74, 77, 3)),
        Gcode::new(Mnemonic::General, 17.0, Span::new(78, 81, 3)),
        Gcode::new(Mnemonic::General, 94.0, Span::new(82, 85, 3)),
        Gcode::new(Mnemonic::General, 80.0, Span::new(86, 89, 3)),
        Gcode::new(Mnemonic::General, 54.0, Span::new(98, 116, 4))
            .with_argument(Word::new('X', -75.0, Span::new(102, 106, 4)))
            .with_argument(Word::new('Y', -75.0, Span::new(107, 111, 4)))
            .with_argument(Word::new('S', 500.0, Span::new(112, 116, 4))),
        Gcode::new(Mnemonic::MachineRoutine, 3.0, Span::new(117, 119, 4)),
        Gcode::new(Mnemonic::General, 43.0, Span::new(142, 153, 5))
            .with_argument(Word::new('Z', 100.0, Span::new(146, 150, 5)))
            .with_argument(Word::new('H', 1.0, Span::new(151, 153, 5))),
        Gcode::new(Mnemonic::General, 1.0, Span::new(162, 168, 6))
            .with_argument(Word::new('Z', 5.0, Span::new(166, 168, 6))),
        Gcode::new(Mnemonic::General, 1.0, Span::new(177, 190, 7))
            .with_argument(Word::new('Z', -20.0, Span::new(181, 185, 7)))
            .with_argument(Word::new('F', 100.0, Span::new(186, 190, 7))),
    ];

    assert_eq!(got, should_be);
}

macro_rules! parse_fixture {
    ($test_name:ident, $filename:expr => $num_codes:expr) => (
        #[test]
        fn $test_name() {
            let src = include_str!(concat!("data/", $filename));
            let num_gcodes = gcode::parse(src).count();

            println!("{}", src);
            assert_eq!(num_gcodes, $num_codes);
        }
        
    )
}

parse_fixture!(parse_program_1, "program_1.gcode" => 24);
parse_fixture!(parse_program_2, "program_2.gcode" => 14);

