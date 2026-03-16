use crate::core::{
    ProgramVisitor,
    lexer::{Token, TokenType, Tokens},
};

/// Opaque state for pausing and resuming a parse.
///
/// When a visitor returns [`ControlFlow::Break`](crate::core::ControlFlow), the
/// parser yields a `ParserState`. Pass that state and the same visitor to
/// [`resume`] to continue from the next block. The state is only valid for the
/// same `src` slice and visitor; do not modify `src` between pause and resume.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParserState {
    current_index: usize,
    current_line: usize,
}

impl ParserState {
    /// State for starting a parse from the beginning of `src`.
    pub const fn empty() -> Self {
        Self::new(0, 0)
    }

    /// State at a specific byte index and line (for use by [`resume`]).
    pub(crate) const fn new(current_index: usize, current_line: usize) -> Self {
        Self {
            current_index,
            current_line,
        }
    }
}

/// Resumes parsing from a saved [`ParserState`].
///
/// Use this when a visitor returned [`ControlFlow::Break`](crate::core::ControlFlow)
/// to continue from the next block. Pass the same `src` and visitor (or one that
/// is logically equivalent). Returns the new state after this chunk of parsing;
/// if the visitor breaks again, pass that state to the next `resume` call.
///
/// # Note
///
/// The implementation assumes that the `src` string ends with a complete line.
/// Behaviour is unspecified if parsing resumes in the middle of a line.
#[must_use]
pub fn resume(
    state: ParserState,
    src: &str,
    _visitor: &mut impl ProgramVisitor,
) -> ParserState {
    let mut tokens = Tokens::new(src, state.current_index, state.current_line);

    {
        let mut tokens = tokens.by_ref().peekable();
        while let Some(Token { kind, .. }) = tokens.next() {
            match kind {
                TokenType::Newline | TokenType::Unknown => {
                    continue;
                },
                _ => todo!(),
            }
        }
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
        BlockVisitor, CommandVisitor, ControlFlow, Diagnostics, Number,
        ProgramVisitor, Span, Value,
    };

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        LineStarted,
        LineNumber(Number, Span),
        Comment(String, Span),
        GeneralCode(Number),
        MiscCode(Number),
        ToolChangeCode(Number),
        ProgramNumber(Number, Span),
        Argument(char, crate::ast::Value, Span),
        UnknownContentError(String, Span),
        Unexpected(String, String, Span),
    }

    struct Recorder<'a>(&'a mut Vec<Event>);

    impl Diagnostics for Recorder<'_> {
        fn emit_unknown_content(&mut self, text: &str, span: Span) {
            self.0
                .push(Event::UnknownContentError(text.to_string(), span));
        }
        fn emit_unexpected(
            &mut self,
            actual: &str,
            expected: &[&str],
            span: Span,
        ) {
            self.0.push(Event::Unexpected(
                actual.to_string(),
                expected.join(", "),
                span,
            ));
        }
    }

    impl ProgramVisitor for Recorder<'_> {
        fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_> {
            self.0.push(Event::LineStarted);
            ControlFlow::Continue(Recorder(self.0))
        }
    }

    impl BlockVisitor for Recorder<'_> {
        fn line_number(&mut self, n: Number, span: Span) {
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
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::GeneralCode(number));
            ControlFlow::Continue(Recorder(self.0))
        }
        fn start_miscellaneous_code(
            &mut self,
            number: Number,
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::MiscCode(number));
            ControlFlow::Continue(Recorder(self.0))
        }
        fn start_tool_change_code(
            &mut self,
            number: Number,
        ) -> ControlFlow<impl CommandVisitor + '_> {
            self.0.push(Event::ToolChangeCode(number));
            ControlFlow::Continue(Recorder(self.0))
        }
    }

    impl CommandVisitor for Recorder<'_> {
        fn argument(&mut self, letter: char, value: Value<'_>, span: Span) {
            self.0.push(Event::Argument(letter, value.into(), span));
        }
    }

    fn parse_and_record(src: &str) -> Vec<Event> {
        let mut events = Vec::new();
        let _ = resume(ParserState::empty(), src, &mut Recorder(&mut events));
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
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
            ]
        );
    }

    #[test]
    fn g_code_with_arguments() {
        let events = parse_and_record("G01 X10 Y-20");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 1,
                    minor: None,
                }),
                Event::Argument(
                    'X',
                    crate::ast::Value::Literal(10.0),
                    sp(4, 3, 0)
                ),
                Event::Argument(
                    'Y',
                    crate::ast::Value::Literal(-20.0),
                    sp(8, 4, 0)
                ),
            ]
        );
    }

    #[test]
    fn comment_semicolon() {
        let events = parse_and_record("; hello world");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
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
                Event::LineStarted,
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
                Event::LineStarted,
                Event::LineNumber(
                    Number {
                        major: 42,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
            ]
        );
    }

    #[test]
    fn program_number_o_code() {
        let events = parse_and_record("O1000");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::ProgramNumber(
                    Number {
                        major: 1000,
                        minor: None,
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
                Event::LineStarted,
                Event::MiscCode(Number {
                    major: 3,
                    minor: None,
                }),
                Event::Argument(
                    'S',
                    crate::ast::Value::Literal(1000.0),
                    sp(3, 5, 0)
                ),
            ]
        );
    }

    #[test]
    fn tool_change_code() {
        let events = parse_and_record("T2");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::ToolChangeCode(Number {
                    major: 2,
                    minor: None,
                }),
            ]
        );
    }

    #[test]
    fn multiple_codes_same_line() {
        let events = parse_and_record("G0 G90 G40 G21");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 0,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 40,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 21,
                    minor: None,
                }),
            ]
        );
    }

    #[test]
    fn two_lines() {
        let events = parse_and_record("G90\nG01 X1");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 1,
                    minor: None,
                }),
                Event::Argument(
                    'X',
                    crate::ast::Value::Literal(1.0),
                    sp(8, 10, 1)
                ),
            ]
        );
    }

    #[test]
    fn decimal_argument() {
        let events = parse_and_record("G01 X1.5 Y-0.25");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 1,
                    minor: None,
                }),
                Event::Argument(
                    'X',
                    crate::ast::Value::Literal(1.5),
                    sp(4, 4, 0)
                ),
                Event::Argument(
                    'Y',
                    crate::ast::Value::Literal(-0.25),
                    sp(9, 6, 0)
                ),
            ]
        );
    }

    #[test]
    fn minor_subcode_g91_1() {
        let events = parse_and_record("G91.1");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 91,
                    minor: Some(1),
                }),
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
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 0,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 21,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 17,
                    minor: None,
                }),
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
            ]
        );
    }

    #[test]
    fn unknown_content_error() {
        let events = parse_and_record("G90 $$%# X10");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
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
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
                Event::Unexpected(
                    "N".into(),
                    "line number".into(),
                    sp(4, 3, 0)
                ),
            ]
        );
    }

    #[test]
    fn letter_without_number_error() {
        let events = parse_and_record("G");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::Unexpected("G".into(), "eof".into(), sp(0, 1, 0)),
            ]
        );
    }

    #[test]
    fn number_without_letter_error() {
        let events = parse_and_record("42");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 90,
                    minor: None,
                }),
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
                Event::LineStarted,
                Event::LineNumber(
                    Number {
                        major: 10,
                        minor: None
                    },
                    sp(0, 3, 0)
                ),
                Event::GeneralCode(Number {
                    major: 0,
                    minor: None,
                }),
                Event::Argument(
                    'X',
                    crate::ast::Value::Literal(1.0),
                    sp(7, 9, 0)
                ),
                Event::Argument(
                    'Y',
                    crate::ast::Value::Literal(2.0),
                    sp(10, 12, 0)
                ),
            ]
        );
    }
}
