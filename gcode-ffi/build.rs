extern crate cbindgen;

use cbindgen::Language;
use cbindgen::{Config, EnumConfig, ParseConfig, RenameRule};
use std::env;
use std::path::PathBuf;

fn main() {
    if running_in_ci() {
        return;
    }

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = crate_dir.parent().unwrap().join("target");
    let header = target_dir.join("gcode.h");

    let cfg = Config {
        documentation: true,
        language: Language::C,
        include_guard: Some("GCODE_H".into()),
        parse: ParseConfig {
            parse_deps: true,
            ..Default::default()
        },
        enumeration: EnumConfig {
            rename_variants: Some(RenameRule::ScreamingSnakeCase),
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::Builder::new()
        .with_config(cfg)
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(header);
}

fn running_in_ci() -> bool {
    let ci_vars = &["CI", "TRAVIS", "APPVEYOR"];

    for var in ci_vars {
        if env::var(var).is_ok() {
            return true;
        }
    }

    false
}
