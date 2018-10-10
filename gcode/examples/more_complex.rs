extern crate gcode;

use gcode::Parser;
use std::env;
use std::fs;
use std::io::{self, Error, Read};

fn main() -> Result<(), Error> {
    let input = read_input()?;

    for (i, block) in Parser::new(&input).enumerate() {
        match block.line_number() {
            Some(n) => println!("Block {} (line N{})", i, n),
            None => println!("Block {}", i),
        }

        for command in block.commands() {
            println!(
                "\t{:?} {} {}",
                command.mnemonic(),
                command.major_number(),
                command
                    .args()
                    .into_iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }

        if !block.comments().is_empty() {
            println!();
            println!("\t{} comments:", block.comments().len());
            for comment in block.comments() {
                println!("\t\t\"{}\"", comment.body);
            }
        }
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
