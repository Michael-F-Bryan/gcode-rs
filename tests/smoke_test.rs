//! Integration tests for the core push-based parser using fixtures from tests/data/.

macro_rules! parser_tests {
    (
        $( $(#[$attr:meta])* $name:ident => $filename:expr),* $(,)?
    ) => {
        $(
            $(#[$attr])*
            #[test]
            fn $name() {
                let src = include_str!(concat!("./data/", $filename));

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
    #[ignore]
    parse_program_3 => "_program_3.gcode",
    #[ignore]
    parse_insulpro_piping => "_Insulpro.Piping.-.115mm.OD.-.40mm.WT.txt",
}
