# GCode

[![Crates.io version](https://img.shields.io/crates/v/gcode.svg)](https://crates.io/crates/gcode)
[![Docs](https://docs.rs/gcode/badge.svg)](https://docs.rs/gcode/)
[![CI](https://github.com/Michael-F-Bryan/gcode-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Michael-F-Bryan/gcode-rs/actions/workflows/ci.yml)

A gcode parser designed for use in `#[no_std]` environments. G-code is the
common name for the programming language used by CNC machines and 3D printers.

Design goals:
- embedded-friendly (no_std / WebAssembly)
- deterministic memory (optional zero allocation via the `core` visitor API)
- error-resistant parsing with diagnostics, and
- *O(n)* performance with no backtracking.

Default features: `alloc`, `serde`. Omit `alloc` for zero-allocation parsing
via the `core` visitor API. Requires Rust 1.85+.

## Getting Started

First, add `gcode` to your dependencies:

```console
$ cargo add gcode
```

Then, you can start using it. Here is a simple example that parses a G-code program using the `alloc` feature:

```rust
fn main() -> Result<(), gcode::Diagnostics> {
    let src = "G90 G00 X10 Y20";
    let program = gcode::parse(src)?;
    assert_eq!(program.blocks.len(), 1);
    Ok(())
}
```

For a visitor-based example, see the `pretty_print_visitor` example. Run it with
`cargo run --example pretty_print_visitor`.

## Useful Links

- [Full API and feature flags][docs.rs] (docs.rs)
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

[docs.rs]: https://docs.rs/gcode/
[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/main/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
[cargo-c]: https://github.com/lu-zero/cargo-c
[etrombly]: https://github.com/etrombly
[gc-y]: https://github.com/etrombly/gcode-yew
[crev]: https://github.com/crev-dev/cargo-crev
