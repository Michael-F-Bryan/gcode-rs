# gcode-rs

[![Crates.io version](https://img.shields.io/crates/v/gcode.svg)](https://crates.io/crates/gcode)
[![Docs](https://docs.rs/gcode/badge.svg)](https://docs.rs/gcode/)
[![Build Status](https://travis-ci.org/Michael-F-Bryan/gcode-rs.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/gcode-rs)

A gcode parser designed for use in `#[no_std]` environments.

For an example of the `gcode` crate in use, see 
[@etrombly][etrombly]'s [`gcode-yew`][gc-y].

## Useful Links

- [The thread that kicked this idea off][thread]
- [Rendered Documentation][docs]
- [NIST GCode Interpreter Spec][nist]

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE_APACHE.md) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE_MIT.md) or
   http://opensource.org/licenses/MIT)

at your option.

It is recommended to always use [cargo-crev][crev] to verify the
trustworthiness of each of your dependencies, including this one.

### Contribution

The intent of this crate is to be free of soundness bugs. The developers will
do their best to avoid them, and welcome help in analyzing and fixing them.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/master/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
[cargo-c]: https://github.com/lu-zero/cargo-c
[etrombly]: https://github.com/etrombly
[gc-y]: https://github.com/etrombly/gcode-yew
[crev]: https://github.com/crev-dev/cargo-crev
