use gcode::{Span, GCode};

macro_rules! smoke_test {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $filename));
            let src = sanitise_input(src);

            let _got: Vec<_> = gcode::parse_with_callbacks(&src, PanicOnError).collect();
        }
    };
}

smoke_test!(program_1, "program_1.gcode");
smoke_test!(program_2, "program_2.gcode");
smoke_test!(program_3, "program_3.gcode");

struct PanicOnError;

impl gcode::Callbacks for PanicOnError {
    fn unknown_content(&mut self, text: &str, span: Span) {
        panic!("Unknown content at {:?}: {}", span, text);
    }
    fn gcode_buffer_overflowed(&mut self, _gcode: GCode) {
        panic!("Buffer overflow");
    }
    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        panic!("Unexlected line number at {:?}: {}", span, line_number);
    }
    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: Span,
    ) {
        panic!("Argument without a command at {:?}: {}{}", span, letter, value);
    }
    fn number_without_a_letter(&mut self, value: &str, span: Span) {
        panic!("Number without a letter at {:?}: {}", span, value);
    }
    fn letter_without_a_number(&mut self, value: &str, span: Span) {
        panic!("Letter without a number at {:?}: {}", span, value);
    }
}

fn sanitise_input(src: &str) -> String {
    let pieces: Vec<&str> = src.split('%').collect();

    match pieces.len() {
        0 => unreachable!(),
        1 => src.to_string(),
        2 => pieces[0].to_string(),
        3 => pieces[1].to_string(),
        _ => panic!(),
    }
}