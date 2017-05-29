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


[thread]:https://users.rust-lang.org/t/g-code-interpreter/10930
[docs]: https://michael-f-bryan.github.io/gcode-rs/
[p3]: https://github.com/Michael-F-Bryan/gcode-rs/blob/master/tests/data/program_3.gcode
[nist]: http://ws680.nist.gov/publication/get_pdf.cfm?pub_id=823374
