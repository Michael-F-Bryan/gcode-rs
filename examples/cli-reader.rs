extern crate gcode;

use std::env;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{self, Error, Read};
use std::process;

fn run() -> Result<(), Error> {
    let mut input = parse_args()?;

    let mut src = String::new();
    input.read_to_string(&mut src)?;

    for code in gcode::parse(&src) {
        println!("{:#?}", code);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {}", e);

        let mut cause = e.cause();

        while let Some(c) = cause {
            eprintln!("\tCaused by: {}", c);
            cause = c.cause();
        }
    }
}

fn parse_args() -> Result<Box<Read>, Error> {
    for arg in env::args() {
        if arg == "-h" || arg == "--help" {
            usage();
        }
    }

    match env::args().nth(1) {
        Some(filename) => Ok(Box::new(File::open(filename)?)),
        None => Ok(Box::new(io::stdin())),
    }
}

fn usage() -> ! {
    eprintln!("USAGE: {} [filename]", env::args().next().unwrap());
    process::exit(1);
}
