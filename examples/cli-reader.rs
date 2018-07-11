extern crate gcode;

use std::env;
use std::process;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{self, Read, Error, ErrorKind, BufReader, BufRead};

fn run() -> Result<(), Error> {
    let input = parse_args()?;

    for line in BufReader::new(input).lines() {
        match line {
            Ok(line) => {
                for code in gcode::parse(&line) {
                    println!("{:#?}", code);
                }
            },
            Err(e) => return Err(e),
        }
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
