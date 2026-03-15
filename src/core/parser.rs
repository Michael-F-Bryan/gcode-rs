use crate::core::{
    ProgramVisitor,
    lexer::{TokenType, Tokens},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParserState {
    current_index: usize,
    current_line: usize,
}

impl ParserState {
    pub const fn empty() -> Self {
        Self::new(0, 0)
    }

    pub const fn new(current_index: usize, current_line: usize) -> Self {
        Self {
            current_index,
            current_line,
        }
    }
}

/// Resume parsing from a given state.
///
/// ## Note
///
/// This implicitly assumes that the `src` string ends with a complete line.
/// Weird things may happen if you start parsing from a partial line.
#[must_use]
pub fn resume(
    state: ParserState,
    src: &str,
    visitor: impl ProgramVisitor,
) -> ParserState {
    let mut tokens = Tokens::new(src, state.current_index, state.current_line);

    {
        let mut tokens = tokens.by_ref().peekable();
    }

    tokens.state()
}

impl Default for ParserState {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
#[allow(refining_impl_trait)]
mod tests {
    use super::*;
    use crate::core::{
        CommandVisitor, ControlFlow, LineVisitor, Number, ProgramVisitor, Span,
    };

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        LineStarted(Span),
        LineNumber(f32, Span),
        Comment(String, Span),
        GeneralCode(Number, Span),
        MiscCode(Number, Span),
        ToolChangeCode(Number, Span),
        ProgramNumber(Number, Span),
        Argument(char, f32, Span),
        UnknownContentError(String, Span),
        UnexpectedLineNumberError(f32, Span),
        LetterWithoutNumberError(String, Span),
        NumberWithoutLetterError(String, Span),
    }

    struct Recorder<'a>(&'a mut Vec<Event>);

    impl ProgramVisitor for Recorder<'_> {
        fn start_line(
            &mut self,
            span: Span,
        ) -> ControlFlow<impl LineVisitor + '_> {
            self.0.push(Event::LineStarted(span));
            ControlFlow::Continue(Recorder(self.0))
        }
    }

    impl LineVisitor for Recorder<'_> {
        fn line_number(&mut self, n: f32, span: Span) {
            self.0.push(Event::LineNumber(n, span));
        }
        fn comment(&mut self, value: &str, span: Span) {
            self.0.push(Event::Comment(value.to_string(), span));
        }
        fn program_number(&mut self, number: Number, span: Span) {
            self.0.push(Event::ProgramNumber(number, span));
        }
        fn start_general_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::GeneralCode(number, span));
            ControlFlow::Continue(Recorder(self.0))
        }
        fn start_miscellaneous_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::MiscCode(number, span));
            ControlFlow::Continue(Recorder(self.0))
        }
        fn start_tool_change_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::ToolChangeCode(number, span));
            ControlFlow::Continue(Recorder(self.0))
        }
        fn unknown_content_error(&mut self, text: &str, span: Span) {
            self.0
                .push(Event::UnknownContentError(text.to_string(), span));
        }
        fn unexpected_line_number_error(&mut self, n: f32, span: Span) {
            self.0.push(Event::UnexpectedLineNumberError(n, span));
        }
        fn letter_without_number_error(&mut self, value: &str, span: Span) {
            self.0
                .push(Event::LetterWithoutNumberError(value.to_string(), span));
        }
        fn number_without_letter_error(&mut self, value: &str, span: Span) {
            self.0
                .push(Event::NumberWithoutLetterError(value.to_string(), span));
        }
    }

    impl CommandVisitor for Recorder<'_> {
        fn argument(&mut self, letter: char, value: f32, span: Span) {
            self.0.push(Event::Argument(letter, value, span));
        }
    }

    fn parse_and_record(src: &str) -> Vec<Event> {
        let mut events = Vec::new();
        let _ = resume(ParserState::empty(), src, Recorder(&mut events));
        events
    }

    fn sp(start: usize, length: usize, line: usize) -> Span {
        Span::new(start, length, line)
    }

    #[test]
    fn empty_input_produces_no_events() {
        let events = parse_and_record("");
        assert_eq!(events, vec![]);
    }

    #[test]
    fn single_g_code_no_args() {
        let events = parse_and_record("G90");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 3, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
            ]
        );
    }

    #[test]
    fn g_code_with_arguments() {
        let events = parse_and_record("G01 X10 Y-20");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 12, 0)),
                Event::GeneralCode(
                    Number {
                        major: 1,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::Argument('X', 10.0, sp(4, 3, 0)),
                Event::Argument('Y', -20.0, sp(8, 4, 0)),
            ]
        );
    }

    #[test]
    fn comment_semicolon() {
        let events = parse_and_record("; hello world");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 13, 0)),
                Event::Comment(" hello world".into(), sp(0, 13, 0)),
            ]
        );
    }

    #[test]
    fn comment_parens() {
        let events = parse_and_record("(Linear / Feed - Absolute)");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 26, 0)),
                Event::Comment(
                    "(Linear / Feed - Absolute)".into(),
                    sp(0, 26, 0)
                ),
            ]
        );
    }

    #[test]
    fn line_number_then_g_code() {
        let events = parse_and_record("N42 G90");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 7, 0)),
                Event::LineNumber(42.0, sp(0, 3, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(4, 3, 0)
                ),
            ]
        );
    }

    #[test]
    fn program_number_o_code() {
        let events = parse_and_record("O1000");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 5, 0)),
                Event::ProgramNumber(
                    Number {
                        major: 1000,
                        minor: None
                    },
                    sp(0, 5, 0)
                ),
            ]
        );
    }

    #[test]
    fn miscellaneous_code_with_arg() {
        let events = parse_and_record("M3 S1000");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 8, 0)),
                Event::MiscCode(
                    Number {
                        major: 3,
                        minor: None
                    },
                    sp(0, 2, 0)
                ),
                Event::Argument('S', 1000.0, sp(3, 5, 0)),
            ]
        );
    }

    #[test]
    fn tool_change_code() {
        let events = parse_and_record("T2");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 2, 0)),
                Event::ToolChangeCode(
                    Number {
                        major: 2,
                        minor: None
                    },
                    sp(0, 2, 0)
                ),
            ]
        );
    }

    #[test]
    fn multiple_codes_same_line() {
        let events = parse_and_record("G0 G90 G40 G21");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 14, 0)),
                Event::GeneralCode(
                    Number {
                        major: 0,
                        minor: None
                    },
                    sp(0, 2, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(3, 3, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 40,
                        minor: None
                    },
                    sp(7, 3, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 21,
                        minor: None
                    },
                    sp(11, 3, 0)
                ),
            ]
        );
    }

    #[test]
    fn two_lines() {
        let events = parse_and_record("G90\nG01 X1");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 3, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::LineStarted(sp(4, 6, 1)),
                Event::GeneralCode(
                    Number {
                        major: 1,
                        minor: None
                    },
                    sp(4, 3, 1)
                ),
                Event::Argument('X', 1.0, sp(8, 10, 1)),
            ]
        );
    }

    #[test]
    fn decimal_argument() {
        let events = parse_and_record("G01 X1.5 Y-0.25");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 15, 0)),
                Event::GeneralCode(
                    Number {
                        major: 1,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::Argument('X', 1.5, sp(4, 4, 0)),
                Event::Argument('Y', -0.25, sp(9, 6, 0)),
            ]
        );
    }

    #[test]
    fn minor_subcode_g91_1() {
        let events = parse_and_record("G91.1");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 5, 0)),
                Event::GeneralCode(
                    Number {
                        major: 91,
                        minor: Some(1)
                    },
                    sp(0, 5, 0)
                ),
            ]
        );
    }

    #[test]
    fn whitespace_only_input() {
        let events = parse_and_record("   \n\t  ");
        assert_eq!(events, vec![]);
    }

    #[test]
    fn no_space_between_words() {
        let events = parse_and_record("G00G21G17G90");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 12, 0)),
                Event::GeneralCode(
                    Number {
                        major: 0,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 21,
                        minor: None
                    },
                    sp(3, 3, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 17,
                        minor: None
                    },
                    sp(6, 9, 0)
                ),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(9, 12, 0)
                ),
            ]
        );
    }

    #[test]
    fn unknown_content_error() {
        let events = parse_and_record("G90 $$%# X10");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 12, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::UnknownContentError("$$%#".into(), sp(4, 4, 0)),
            ]
        );
    }

    #[test]
    fn unexpected_line_number_error() {
        let events = parse_and_record("G90 N42");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 7, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::UnexpectedLineNumberError(42.0, sp(4, 3, 0)),
            ]
        );
    }

    #[test]
    fn letter_without_number_error() {
        let events = parse_and_record("G");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 1, 0)),
                Event::LetterWithoutNumberError("G".into(), sp(0, 1, 0)),
            ]
        );
    }

    #[test]
    fn number_without_letter_error() {
        let events = parse_and_record("42");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 2, 0)),
                Event::NumberWithoutLetterError("42".into(), sp(0, 2, 0)),
            ]
        );
    }

    #[test]
    fn code_then_comment_same_line() {
        let events = parse_and_record("G90 ; absolute mode");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 19, 0)),
                Event::GeneralCode(
                    Number {
                        major: 90,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::Comment(" absolute mode".into(), sp(4, 19, 0)),
            ]
        );
    }

    #[test]
    fn regression_fixed_snippet_event_sequence() {
        let events = parse_and_record("N10 G0 X1 Y2");
        assert_eq!(
            events,
            vec![
                Event::LineStarted(sp(0, 12, 0)),
                Event::LineNumber(10.0, sp(0, 3, 0)),
                Event::GeneralCode(
                    Number {
                        major: 0,
                        minor: None
                    },
                    sp(4, 6, 0)
                ),
                Event::Argument('X', 1.0, sp(7, 9, 0)),
                Event::Argument('Y', 2.0, sp(10, 12, 0)),
            ]
        );
    }
}
