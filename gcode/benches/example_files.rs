#![feature(test)]

extern crate test;

use test::Bencher;

macro_rules! bench {
    ($name:ident) => {
        #[bench]
        #[allow(non_snake_case)]
        fn $name(b: &mut Bencher) {
            let src = include_str!(concat!("../tests/data/", stringify!($name), ".gcode"));
            b.bytes = src.len() as u64;

            b.iter(|| gcode::parse(src).count());
        }
    };
}

bench!(program_1);
bench!(program_2);
bench!(program_3);
bench!(PI_octcat);
