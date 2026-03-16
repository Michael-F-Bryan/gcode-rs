use crate::core::{
    BlockVisitor, CommandVisitor, ControlFlow, Number, ProgramVisitor, Span, Value,
    lexer::{Token, TokenType, Tokens},
};

/// Peekable token stream that exposes the current parse position (for resume state).
struct TokenStream<'a, 'src> {
    peeked: Option<Token<'src>>,
    tokens: &'a mut Tokens<'src>,
}

impl<'a, 'src> TokenStream<'a, 'src> {
    fn new(tokens: &'a mut Tokens<'src>) -> Self {
        Self {
            peeked: None,
            tokens,
        }
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        if self.peeked.is_none() {
            self.peeked = self.tokens.next();
        }
        self.peeked.as_ref()
    }

    fn next(&mut self) -> Option<Token<'src>> {
        if self.peeked.is_some() {
            self.peeked.take()
        } else {
            self.tokens.next()
        }
    }

    fn state_ref(&self) -> ParserState {
        self.peeked
            .as_ref()
            .map(|t| ParserState::new(t.span.start, t.span.line))
            .unwrap_or_else(|| self.tokens.state_ref())
    }
}

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

/// Parses a g-code number string (e.g. `"90"` or `"91.1"`) into [`Number`].
fn parse_number(value: &str) -> Option<Number> {
    let (major_str, minor_str) = match value.split_once('.') {
        None => (value, None),
        Some((maj, min)) => (maj, Some(min)),
    };
    let major = major_str.parse::<u16>().ok()?;
    let minor = match minor_str {
        Some(s) => Some(s.parse::<u16>().ok()?),
        None => None,
    };
    Some(Number { major, minor })
}

/// Builds a literal argument value from an optional sign and number string.
fn literal_value(sign: i8, number_value: &str) -> Option<Value<'_>> {
    let n: f32 = number_value.parse().ok()?;
    let n = match sign {
        -1 => -n,
        _ => n,
    };
    Some(Value::Literal(n))
}

/// Parses command arguments (Letter + optional sign + Number) until recovery set.
/// Returns command span (from command start through last argument) for `end_command`.
fn parse_command_arguments<'a, 'src>(
    stream: &mut TokenStream<'a, 'src>,
    cmd_visitor: &mut impl CommandVisitor,
    command_start_span: Span,
) -> Span {
    let mut last_span = command_start_span;
    loop {
        let peek = stream.peek();
        let letter_token = match peek {
            Some(Token {
                kind: TokenType::Letter,
                value,
                span,
            }) if value.len() == 1 => {
                let c = value.chars().next().unwrap();
                if matches!(c, 'G' | 'M' | 'T' | 'N' | 'O') {
                    break;
                }
                Some((*span, c))
            }
            Some(Token {
                kind: TokenType::Letter,
                ..
            }) => break,
            Some(_) => break,
            None => break,
        };
        let (letter_span, letter_c) = match letter_token {
            Some(s) => s,
            None => break,
        };
        let _ = stream.next();
        let (sign, number_token) = match stream.peek() {
            Some(Token {
                kind: TokenType::MinusSign,
                ..
            }) => {
                let _ = stream.next();
                (-1i8, stream.next())
            }
            Some(Token {
                kind: TokenType::PlusSign,
                ..
            }) => {
                let _ = stream.next();
                (1i8, stream.next())
            }
            _ => (0i8, stream.next()),
        };
        let number_token = match number_token {
            Some(Token {
                kind: TokenType::Number,
                value,
                span,
            }) => (value, span),
            _ => {
                let mut buf = [0u8; 4];
                let s = letter_c.encode_utf8(&mut buf);
                cmd_visitor.diagnostics().emit_unexpected(s, &["eof"], letter_span);
                break;
            }
        };
        let (num_val, num_span) = number_token;
        let value = match literal_value(sign, num_val) {
            Some(v) => v,
            None => {
                cmd_visitor
                    .diagnostics()
                    .emit_unexpected(num_val, &["number"], num_span);
                break;
            }
        };
        let arg_span = Span::new(
            letter_span.start,
            num_span.end() - letter_span.start,
            letter_span.line,
        );
        last_span = Span::new(
            command_start_span.start,
            arg_span.end() - command_start_span.start,
            command_start_span.line,
        );
        cmd_visitor.argument(letter_c, value, arg_span);
    }
    last_span
}

/// Parses one line (block): optional N/O/comment, then zero or more G/M/T commands.
/// Consumes tokens up to and including the terminating Newline (or EOF).
/// Returns Break if the visitor asked to pause; otherwise Continue and the line span.
fn parse_line<'a, 'src, B: BlockVisitor>(
    stream: &mut TokenStream<'a, 'src>,
    block: &mut B,
    mut current: Token<'src>,
    line_start_span: Span,
) -> (ControlFlow<()>, Span) {
    let mut line_span;
    let mut seen_command = false;
    loop {
        line_span = Span::new(
            line_start_span.start,
            current.span.end() - line_start_span.start,
            line_start_span.line,
        );
        match current.kind {
            TokenType::Newline => {
                return (ControlFlow::Continue(()), line_span);
            }
            TokenType::Letter => {
                let c = current.value.chars().next().unwrap_or('\0');
                if current.value.len() != 1 {
                    block.diagnostics().emit_unexpected(current.value, &["single letter"], current.span);
                    match stream.next() {
                        Some(t) => current = t,
                        None => return (ControlFlow::Continue(()), line_span),
                    }
                    continue;
                }
                match c {
                    'N' => {
                        if seen_command {
                            let num_tok = stream.next();
                            let span = match &num_tok {
                                Some(Token { span: s, .. }) => Span::new(
                                    current.span.start,
                                    s.end() - current.span.start,
                                    current.span.line,
                                ),
                                None => current.span,
                            };
                            block.diagnostics().emit_unexpected("N", &["line number"], span);
                        } else {
                            let num_tok = stream.next();
                            match num_tok {
                                Some(Token {
                                    kind: TokenType::Number,
                                    value,
                                    span,
                                }) => {
                                    if let Some(n) = parse_number(value) {
                                        let n_span = Span::new(
                                            current.span.start,
                                            span.end() - current.span.start,
                                            current.span.line,
                                        );
                                        block.line_number(n, n_span);
                                    }
                                }
                                _ => {
                                    block.diagnostics().emit_unexpected("N", &["eof"], current.span);
                                }
                            }
                        }
                    }
                    'O' => {
                        if seen_command {
                            let num_tok = stream.next();
                            let span = match &num_tok {
                                Some(Token { span: s, .. }) => Span::new(
                                    current.span.start,
                                    s.end() - current.span.start,
                                    current.span.line,
                                ),
                                None => current.span,
                            };
                            block.diagnostics().emit_unexpected("O", &["program number"], span);
                        } else {
                            let num_tok = stream.next();
                            match num_tok {
                                Some(Token {
                                    kind: TokenType::Number,
                                    value,
                                    span,
                                }) => {
                                    if let Some(n) = parse_number(value) {
                                        let o_span = Span::new(
                                            current.span.start,
                                            span.end() - current.span.start,
                                            current.span.line,
                                        );
                                        block.program_number(n, o_span);
                                    }
                                }
                                _ => {
                                    block.diagnostics().emit_unexpected("O", &["eof"], current.span);
                                }
                            }
                        }
                    }
                    'G' => {
                        seen_command = true;
                        let num_tok = stream.next();
                        match num_tok {
                            Some(Token {
                                kind: TokenType::Number,
                                value,
                                span,
                            }) => {
                                if let Some(n) = parse_number(value) {
                                    let cmd_span = Span::new(
                                        current.span.start,
                                        span.end() - current.span.start,
                                        current.span.line,
                                    );
                                    match block.start_general_code(n) {
                                        ControlFlow::Break(()) => {
                                            return (ControlFlow::Break(()), line_span);
                                        }
                                        ControlFlow::Continue(mut cmd) => {
                                            let end_span = parse_command_arguments(stream, &mut cmd, cmd_span);
                                            cmd.end_command(end_span);
                                        }
                                    }
                                }
                            }
                            _ => {
                                block.diagnostics().emit_unexpected("G", &["eof"], current.span);
                            }
                        }
                    }
                    'M' => {
                        seen_command = true;
                        let num_tok = stream.next();
                        match num_tok {
                            Some(Token {
                                kind: TokenType::Number,
                                value,
                                span,
                            }) => {
                                if let Some(n) = parse_number(value) {
                                    let cmd_span = Span::new(
                                        current.span.start,
                                        span.end() - current.span.start,
                                        current.span.line,
                                    );
                                    match block.start_miscellaneous_code(n) {
                                        ControlFlow::Break(()) => {
                                            return (ControlFlow::Break(()), line_span);
                                        }
                                        ControlFlow::Continue(mut cmd) => {
                                            let end_span = parse_command_arguments(stream, &mut cmd, cmd_span);
                                            cmd.end_command(end_span);
                                        }
                                    }
                                }
                            }
                            _ => {
                                block.diagnostics().emit_unexpected("M", &["eof"], current.span);
                            }
                        }
                    }
                    'T' => {
                        seen_command = true;
                        let num_tok = stream.next();
                        match num_tok {
                            Some(Token {
                                kind: TokenType::Number,
                                value,
                                span,
                            }) => {
                                if let Some(n) = parse_number(value) {
                                    let cmd_span = Span::new(
                                        current.span.start,
                                        span.end() - current.span.start,
                                        current.span.line,
                                    );
                                    match block.start_tool_change_code(n) {
                                        ControlFlow::Break(()) => {
                                            return (ControlFlow::Break(()), line_span);
                                        }
                                        ControlFlow::Continue(mut cmd) => {
                                            let end_span = parse_command_arguments(stream, &mut cmd, cmd_span);
                                            cmd.end_command(end_span);
                                        }
                                    }
                                }
                            }
                            _ => {
                                block.diagnostics().emit_unexpected("T", &["eof"], current.span);
                            }
                        }
                    }
                    _ => {
                        block.diagnostics().emit_unexpected(
                            current.value,
                            &["G, M, T, N, O, comment"],
                            current.span,
                        );
                    }
                }
            }
            TokenType::Comment => {
                let value = current
                    .value
                    .strip_prefix(';')
                    .unwrap_or(current.value);
                block.comment(value, current.span);
            }
            TokenType::Number => {
                block.diagnostics().emit_unexpected(current.value, &["letter"], current.span);
            }
            TokenType::Unknown => {
                block.diagnostics().emit_unknown_content(current.value, current.span);
            }
            TokenType::Slash | TokenType::MinusSign | TokenType::PlusSign => {
                block.diagnostics().emit_unexpected(current.value, &["word or comment"], current.span);
            }
        }
        match stream.next() {
            Some(t) => current = t,
            None => return (ControlFlow::Continue(()), line_span),
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
    visitor: &mut impl ProgramVisitor,
) -> ParserState {
    let mut tokens = Tokens::new(src, state.current_index, state.current_line);
    let mut return_state = None;
    {
        let mut stream = TokenStream::new(&mut tokens);
        loop {
            while matches!(
                stream.peek(),
                Some(Token {
                    kind: TokenType::Newline,
                    ..
                })
            ) {
                let _ = stream.next();
            }
            let first = match stream.next() {
                None => break,
                Some(t) => t,
            };
            if first.kind == TokenType::Newline {
                continue;
            }
            let line_start_span = first.span;
            match visitor.start_block() {
                ControlFlow::Break(()) => {
                    return_state = Some(stream.state_ref());
                    break;
                }
                ControlFlow::Continue(mut block) => {
                    let (flow, line_span) =
                        parse_line(&mut stream, &mut block, first, line_start_span);
                    if flow.is_break() {
                        return_state = Some(stream.state_ref());
                        break;
                    }
                    block.end_line(line_span);
                }
            }
        }
    }
    return_state.unwrap_or_else(|| tokens.state())
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
                    sp(8, 2, 1)
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
                Event::Unexpected(
                    "X".into(),
                    "G, M, T, N, O, comment".into(),
                    sp(9, 1, 0)
                ),
                Event::Unexpected("10".into(), "letter".into(), sp(10, 2, 0)),
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
                Event::Unexpected("42".into(), "letter".into(), sp(0, 2, 0)),
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
                    sp(7, 2, 0)
                ),
                Event::Argument(
                    'Y',
                    crate::ast::Value::Literal(2.0),
                    sp(10, 2, 0)
                ),
            ]
        );
    }
}
