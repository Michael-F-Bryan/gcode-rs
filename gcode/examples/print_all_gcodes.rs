extern crate gcode;

use std::env;
use std::fs;
use std::io::{self, Error, Read};

fn main() -> Result<(), Error> {
    let input = read_input()?;

    for command in gcode::parse(&input) {
        println!("{:?}", command);
    }

    Ok(())
}

fn read_input() -> Result<String, Error> {
    match env::args().nth(1) {
        Some(filename) => fs::read_to_string(filename),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}
