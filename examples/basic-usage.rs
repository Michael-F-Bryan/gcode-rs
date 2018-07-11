extern crate gcode;

use gcode::Mnemonic;

const PROGRAM_1: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/program_1.gcode"));

fn main() {
    let mut g_codes = 0;
    let mut m_codes = 0;
    let mut total_args = 0;
    let mut x_arguments = 0;

    for block in gcode::parse(PROGRAM_1) {
        println!("{:#?}", block);

        match block.mnemonic() {
            Mnemonic ::General => g_codes += 1,
            Mnemonic::MachineRoutine => m_codes += 1,
            _ => {}
        }

        total_args += block.args().len();

        if let Some(
    }

    println!("G-codes: {}", g_codes);
    println!("M-codes: {}", m_codes);
    println!("Total Arguments: {}", total_args);
}
