# gcode-rs

[![Crates.io version](https://img.shields.io/crates/v/gcode.svg)](https://crates.io/crates/gcode)
[![Docs](https://docs.rs/gcode/badge.svg)](https://docs.rs/gcode/)
[![License](http://img.shields.io/:license-MIT-blue.svg)](http://doge.mit-license.org)
[![Build Status](https://travis-ci.org/Michael-F-Bryan/gcode-rs.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/gcode-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/1b9pank3tu0oaoy7?svg=true)](https://ci.appveyor.com/project/Michael-F-Bryan/gcode-rs)

A gcode parser designed for use in `#[no_std]` environments.

## Getting Started

The parser API itself is quite minimal, consisting of a single `parse()`
function that returns a stream of `Gcode` structs.

```rust
extern crate gcode;

fn main() {
    let src = "O1000
        T1 M6
        (Linear / Feed - Absolute)
        G0 G90 G40 G21 G17 G94 G80
        G54 X-75 Y-75 S500 M3  (Position 6)
        G43 Z100 H1
        G01 Z5
        N42 G01 Z-20 F100";

    for instruction in gcode::parse(src) {
        print!("{:?} {}", instruction.mnemonic(), instruction.major_number());

        if let Some(minor) = instruction.minor_number() {
            print!(".{}", minor);
        }

        for arg in instruction.args() {
            print!(" {}{}", arg.letter, arg.value);
        }

        println!("\t(line {})", instruction.span().source_line);
    }
}
```

## C API

This crate can also be used like a normal C library. This is done using the 
[cargo-c].

## Useful Links

- [The thread that kicked this idea off][thread]
- [Rendered Documentation][docs]
- [NIST GCode Interpreter Spec][nist]


[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/master/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
[cargo-c]: https://github.com/lu-zero/cargo-c
