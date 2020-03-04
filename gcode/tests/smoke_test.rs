use gcode::{GCode, Mnemonic, Span, Word};

macro_rules! smoke_test {
    ($name:ident, $filename:expr) => {
        #[test]
        #[cfg(feature = "std")]
        fn $name() {
            let src = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/",
                $filename
            ));
            let src = sanitise_input(src);

            let _got: Vec<_> =
                gcode::parse_with_callbacks(&src, PanicOnError).collect();
        }
    };
}

smoke_test!(program_1, "program_1.gcode");
smoke_test!(program_2, "program_2.gcode");
smoke_test!(program_3, "program_3.gcode");
smoke_test!(pi_octcat, "PI_octcat.gcode");
smoke_test!(pi_rustlogo, "PI_rustlogo.gcode");
smoke_test!(insulpro_piping, "Insulpro.Piping.-.115mm.OD.-.40mm.WT.txt");

#[test]
#[ignore]
fn expected_program_2_output() {
    // N10 T2 M3 S447 F80
    // N20 G0 X112 Y-2
    // ;N30 Z-5
    // N40 G41
    // N50 G1 X95 Y8 M8
    // ;N60 X32
    // ;N70 X5 Y15
    // ;N80 Y52
    // N90 G2 X15 Y62 I10 J0
    // N100 G1 X83
    // N110 G3 X95 Y50 I12 J0
    // N120 G1 Y-12
    // N130 G40
    // N140 G0 Z100 M9
    // ;N150 X150 Y150
    // N160 M30

    let src = include_str!("data/program_2.gcode");

    let got: Vec<_> = gcode::parse_with_callbacks(src, PanicOnError).collect();

    // total lines
    assert_eq!(got.len(), 20);
    // check lines without any comments
    assert_eq!(got.iter().filter(|l| l.comments().is_empty()).count(), 11);

    let gcodes: Vec<_> = got.iter().flat_map(|l| l.gcodes()).cloned().collect();
    let expected = vec![
        GCode::new(Mnemonic::ToolChange, 2.0, Span::PLACEHOLDER),
        GCode::new(Mnemonic::Miscellaneous, 3.0, Span::PLACEHOLDER)
            .with_argument(Word::new('S', 447.0, Span::PLACEHOLDER))
            .with_argument(Word::new('F', 80.0, Span::PLACEHOLDER)),
    ];
    pretty_assertions::assert_eq!(gcodes, expected);
}

struct PanicOnError;

impl gcode::Callbacks for PanicOnError {
    fn unknown_content(&mut self, text: &str, span: Span) {
        panic!("Unknown content at {:?}: {}", span, text);
    }

    fn gcode_buffer_overflowed(
        &mut self,
        _mnemonic: Mnemonic,
        _major_number: u32,
        _minor_number: u32,
        _arguments: &[Word],
        _span: Span,
    ) {
        panic!("Buffer overflow");
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        panic!("Unexpected line number at {:?}: {}", span, line_number);
    }

    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: Span,
    ) {
        panic!(
            "Argument without a command at {:?}: {}{}",
            span, letter, value
        );
    }

    fn number_without_a_letter(&mut self, value: &str, span: Span) {
        panic!("Number without a letter at {:?}: {}", span, value);
    }

    fn letter_without_a_number(&mut self, value: &str, span: Span) {
        panic!("Letter without a number at {:?}: {}", span, value);
    }
}

#[allow(dead_code)]
fn sanitise_input(src: &str) -> String {
    let mut src = src.to_string();
    let callbacks = [handle_percent, ignore_message_lines];

    for cb in &callbacks {
        src = cb(&src);
    }

    src
}

#[allow(dead_code)]
fn handle_percent(src: &str) -> String {
    let pieces: Vec<&str> = src.split('%').collect();

    match pieces.len() {
        0 => unreachable!(),
        1 => src.to_string(),
        2 => pieces[0].to_string(),
        3 => pieces[1].to_string(),
        _ => panic!(),
    }
}

#[allow(dead_code)]
fn ignore_message_lines(src: &str) -> String {
    // "M117 Printing..." uses string arguments, not the normal char-float word
    let blacklist = ["M117"];

    src.lines()
        .filter(|line| blacklist.iter().all(|word| !line.contains(word)))
        .collect::<Vec<_>>()
        .join("\n")
}
