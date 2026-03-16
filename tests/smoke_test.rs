//! Integration tests for the core push-based parser using fixtures from tests/data/.

#![allow(refining_impl_trait)]

fn load_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

macro_rules! parser_tests {
    (
        $($name:ident => $filename:expr),* $(,)?
    ) => {
        $(
            #[test]
            fn $name() {
                let src = load_fixture($filename);

                gcode::core::parse(&src, &mut gcode::core::Noop);

                #[cfg(feature = "alloc")] {
                    let program = gcode::parse(&src).unwrap();
                    insta::assert_debug_snapshot!(program);
                }
            }
        )*
    };
}

parser_tests! {
    parse_program_1 => "program_1.gcode",
    parse_program_2 => "program_2.gcode",
    parse_program_3 => "program_3.gcode",
    parse_octocat => "PI_octcat.gcode",
    parse_rustlogo => "PI_rustlogo.gcode",
    parse_insulpro_piping => "Insulpro.Piping.-.115mm.OD.-.40mm.WT.txt",
}
