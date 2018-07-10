extern crate arrayvec;
extern crate gcode;

use arrayvec::ArrayVec;
use gcode::{Gcode, Mnemonic, Span, Word};

//const PROGRAM_1: &str = include_str!("data/program_1.gcode");

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
        Gcode {
            mnemonic: Mnemonic::ProgramNumber,
            number: 1000.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 0,
                end: 5,
                source_line: 0,
            },
        },
        Gcode {
            mnemonic: Mnemonic::ToolChange,
            number: 1.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 14,
                end: 16,
                source_line: 1,
            },
        },
        Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: 6.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 17,
                end: 19,
                source_line: 1,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 0.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 63,
                end: 65,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 90.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 66,
                end: 69,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 40.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 70,
                end: 73,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 21.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 74,
                end: 77,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 17.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 78,
                end: 81,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 94.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 82,
                end: 85,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 80.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 86,
                end: 89,
                source_line: 3,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 54.0,
            arguments: vec![
                Word {
                    span: Span {
                        start: 102,
                        end: 106,
                        source_line: 4,
                    },
                    letter: 'X',
                    number: -75.0,
                },
                Word {
                    span: Span {
                        start: 107,
                        end: 111,
                        source_line: 4,
                    },
                    letter: 'Y',
                    number: -75.0,
                },
                Word {
                    span: Span {
                        start: 112,
                        end: 116,
                        source_line: 4,
                    },
                    letter: 'S',
                    number: 500.0,
                },
            ].into_iter()
                .collect(),
            span: Span {
                start: 98,
                end: 116,
                source_line: 4,
            },
        },
        Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: 3.0,
            arguments: ArrayVec::default(),
            span: Span {
                start: 117,
                end: 119,
                source_line: 4,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 43.0,
            arguments: vec![
                Word {
                    span: Span {
                        start: 146,
                        end: 150,
                        source_line: 5,
                    },
                    letter: 'Z',
                    number: 100.0,
                },
                Word {
                    span: Span {
                        start: 151,
                        end: 153,
                        source_line: 5,
                    },
                    letter: 'H',
                    number: 1.0,
                },
            ].into_iter()
                .collect(),
            span: Span {
                start: 142,
                end: 153,
                source_line: 5,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 1.0,
            arguments: vec![Word {
                span: Span {
                    start: 166,
                    end: 168,
                    source_line: 6,
                },
                letter: 'Z',
                number: 5.0,
            }].into_iter()
                .collect(),
            span: Span {
                start: 162,
                end: 168,
                source_line: 6,
            },
        },
        Gcode {
            mnemonic: Mnemonic::General,
            number: 1.0,
            arguments: vec![
                Word {
                    span: Span {
                        start: 181,
                        end: 185,
                        source_line: 7,
                    },
                    letter: 'Z',
                    number: -20.0,
                },
                Word {
                    span: Span {
                        start: 186,
                        end: 190,
                        source_line: 7,
                    },
                    letter: 'F',
                    number: 100.0,
                },
            ].into_iter()
                .collect(),
            span: Span {
                start: 177,
                end: 190,
                source_line: 7,
            },
        },
    ];

    assert_eq!(got, should_be);
}
