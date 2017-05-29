# gcode-rs

[![Build Status](https://travis-ci.org/Michael-F-Bryan/gcode-rs.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/gcode-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/1b9pank3tu0oaoy7?svg=true)](https://ci.appveyor.com/project/Michael-F-Bryan/gcode-rs)


A gcode parser designed to turn a stream of characters into valid gcode
instructions.

> **Note:** For now this crate uses `f32` to represent all numbers. If you
> are wanting to use it on an architecture which doesn't support floats, let
> me know as a comment on
> [this issue](https://github.com/Michael-F-Bryan/gcode-rs/issues/7) and I'll
> see what I can do to help.


## Useful Links

- [The thread that kicked this idea off][thread]
- [Rendered Documentation][docs]
- [NIST GCode Interpreter Spec][nist]


## Contrived Benchmarks

Here are a couple benchmarks using [tests/data/program_3.gcode][p3] as source
code. Note that this was carried out on my Arch Linux laptop with 8GB of RAM
and 4 cores (i7).

```bash
$ wc tests/data/program_3.gcode
411   413 11874 tests/data/program_3.gcode

$ cargo bench
    bench_parse_program_3 ... bench:     216,289 ns/iter (+/- 9,230)
    lex_program_3         ... bench:     144,989 ns/iter (+/- 2,349)
```

If you do the calcs, this adds up to about 12 ns/character for lexing and 18.5
ns/character for both lexing and parsing on my machine.

As usual, take these numbers with a massive pinch of salt.

## Contributing

Contributing to this project is really easy! Because I'm wanting to make the
high level representation for gcodes strongly typed, this requires adding
each and every G and M code, as well as ensuring their invariants are upheld
(for example, providing **both** a radius and a centre point in a G02 is
an error).

Here's the process I followed when adding support for `G04` (dwell).

1. Go to the bottom of `src/high_level.rs` and add a new test for the G code
   with all the valid arguments provided.

```rust
g_code_test!(g_04, (4, &[Argument::new(ArgumentKind::P, 100.0)])
                => GCode::G04 { seconds: 100.0 });
```

2. Check the [NIST][nist] spec (section 3.4.4) for any error conditions and add
   tests for those

```rust
g_code_error!(g_04_requires_a_duration, (4, &[]));
g_code_error!(g_04_duration_cant_be_negative, (4, &[Argument::new(ArgumentKind::P, -1.23)]));
```

3. Add that variant to the `GCode` enum

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GCode {
    /// Rapid Linear Motion
    G00 { to: Point, feed_rate: Option<f32> },
    ...
    /// Dwell - wait for a number of seconds
    G04 { seconds: f32 },
}
```

4. Run the tests and make sure they fail (if they don't something went very
   wrong and you should submit a bug report). The error message will tell you
   roughly where in the file to go to next.

```bash
$ cargo test
    ...

---- high_level::tests::g_04 stdout ----
        thread 'high_level::tests::g_04' panicked at 'G Code not yet supported: 4', src/high_level.rs:79
note: Run with `RUST_BACKTRACE=1` for a backtrace.

    ...
```

5. Go to that line and add a case for that G code number, making sure to add
   appropriate error checking.


```rust
        4 => {
            if let Some(secs) = arg_reader.seconds {
                if secs < 0.0 {
                    Err(Error::InvalidCommand("Dwell duration cannot be negative"))
                } else {
                    Ok(GCode::G04 { seconds: secs })
                }
            } else {
                Err(Error::InvalidCommand("Must provide a dwell duration"))
            }
        }
```

6. Make a pull request.
7. ???
8. Profit!!!



[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/master/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
