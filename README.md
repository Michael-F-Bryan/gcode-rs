# gcode-rs

[![Crates.io version](https://img.shields.io/crates/v/gcode.svg)](https://crates.io/crates/gcode)
[![Docs](https://docs.rs/gcode/badge.svg)](https://docs.rs/gcode/)
[![License](http://img.shields.io/:license-MIT-blue.svg)](http://doge.mit-license.org)
[![Build Status](https://travis-ci.org/Michael-F-Bryan/gcode-rs.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/gcode-rs)

A gcode parser designed for use in `#[no_std]` environments.

For an example of the `gcode` crate in use, see 
[@etrombly][etrombly]'s [`gcode-yew`][gc-y].

## Useful Links

- [The thread that kicked this idea off][thread]
- [Rendered Documentation][docs]
- [NIST GCode Interpreter Spec][nist]


[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/master/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
[cargo-c]: https://github.com/lu-zero/cargo-c
[etrombly]: https://github.com/etrombly
[gc-y]: https://github.com/etrombly/gcode-yew
