extern crate cbindgen;

use std::env;
use std::path::PathBuf;
use cbindgen::{Config, Language};

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = crate_dir.join("src").join("ffi.rs");

    let cfg = Config {
        language: Language::C,
        include_guard: Some(String::from("GCODE_H")),
        ..Default::default()
    };


    cbindgen::Builder::new()
        .with_config(cfg)
        .with_crate(&crate_dir)
        .generate()
        .unwrap()
        .write_to_file(crate_dir.join("gcode.h"));
}
