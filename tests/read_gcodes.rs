extern crate gcode;

use gcode::Mnemonic;

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

    let mut lines = gcode::parse(src);

    let first_line = lines.next().unwrap();
    assert_eq!(first_line.mnemonic, Mnemonic::ProgramNumber);
    assert_eq!(first_line.number, 1000);
    
    let tool_change = lines.next().unwrap();
    assert_eq!(tool_change.mnemonic, Mnemonic::ToolChange);
    assert_eq!(tool_change.number, 1);

    let m6 = lines.next().unwrap();
    assert_eq!(m6.mnemonic, Mnemonic::MachineRoutine);
    assert_eq!(m6.number, 6);

    let g0 = lines.next().unwrap();
    assert_eq!(g0.mnemonic, Mnemonic::General);
    assert_eq!(g0.number, 0);
}
