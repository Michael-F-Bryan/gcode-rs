//! Resilient LL-style parser for g-code. Each grammar rule is a small function that
//! drives the visitor; recovery and follow sets control when we skip vs break.
//!
//! Grammar (conceptual):
//!
//! - Program = Block*
//! - Block   = (line_number | program_number | comment | command)* Newline?
//! - Command = ('G'|'M'|'T') number argument*
//! - Argument = letter ('+'|'-')? number
//!
//! First sets and recovery sets are defined per loop so we know when to parse,
//! skip with diagnostic, or break to the outer loop.

use crate::core::{
    BlockVisitor, CommandVisitor, ControlFlow, Number, ProgramVisitor, Span,
    TokenType, Value,
    lexer::{Token, Tokens},
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

// ---------- Token helpers (no allocation) ----------

fn at(tokens: &mut Tokens<'_>, kind: TokenType) -> bool {
    tokens.peek_token().map(|t| t.kind == kind).unwrap_or(false)
}

/// True if the next token is a single letter equal to `c`.
#[allow(dead_code)]
fn at_letter(tokens: &mut Tokens<'_>, c: char) -> bool {
    match tokens.peek_token() {
        Some(t) if t.kind == TokenType::Letter && t.value.len() == 1 => {
            t.value.chars().next() == Some(c)
        },
        _ => false,
    }
}

/// True if the next token is in the given set of token types.
fn at_any_kind(tokens: &mut Tokens<'_>, set: &[TokenType]) -> bool {
    match tokens.peek_token() {
        Some(t) => set.contains(&t.kind),
        None => false,
    }
}

/// Block-level recovery set: tokens that end the current line (break inner loop).
const BLOCK_FOLLOW: &[TokenType] = &[TokenType::Newline];

/// Argument-level recovery: tokens that end the argument list (break so we don't consume the next command).
/// Letter G/M/T/N/O, Comment, Newline. We check letter in the loop.
fn at_block_follow(tokens: &mut Tokens<'_>) -> bool {
    at_any_kind(tokens, BLOCK_FOLLOW) || tokens.peek_token().is_none()
}

/// First set for a block item: something we can parse as line number, program number, comment, or command.
#[allow(dead_code)]
fn is_block_item_start(tokens: &mut Tokens<'_>) -> bool {
    match tokens.peek_token() {
        None => false,
        Some(Token { kind, .. }) => matches!(
            kind,
            TokenType::Letter
                | TokenType::Comment
                | TokenType::Unknown
                | TokenType::Number
                | TokenType::Slash
                | TokenType::MinusSign
                | TokenType::PlusSign
        ),
    }
}

/// First set for an argument: single letter that is not N, O, G, M, T.
fn is_argument_start(tokens: &mut Tokens<'_>) -> Option<char> {
    let t = tokens.peek_token()?;
    if t.kind != TokenType::Letter || t.value.len() != 1 {
        return None;
    }
    let c = t.value.chars().next()?;
    if matches!(c, 'N' | 'O' | 'G' | 'M' | 'T') {
        return None;
    }
    Some(c)
}

// ---------- Number / value helpers ----------

fn parse_number(value: &str) -> Option<Number> {
    let (major_str, minor_str) = match value.split_once('.') {
        None => (value, None),
        Some((maj, min)) => (maj, Some(min)),
    };
    let major = major_str.parse::<u16>().ok()?;
    let minor = match minor_str {
        Some(s) => s.parse::<u16>().ok(),
        None => None,
    };
    Some(Number { major, minor })
}

fn literal_value(sign: i8, number_value: &str) -> Option<Value<'_>> {
    let n: f32 = number_value.parse().ok()?;
    let n = match sign {
        -1 => -n,
        _ => n,
    };
    Some(Value::Literal(n))
}

// ---------- Grammar: argument list ----------

/// Argument = letter ('+'|'-')? number
/// Recovery: G/M/T/N/O, Comment, Newline, EOF → break (don't consume).
fn parse_argument<C: CommandVisitor>(
    tokens: &mut Tokens<'_>,
    cmd: &mut C,
    _line_start: Span,
) -> Option<Span> {
    let letter_c = is_argument_start(tokens)?;
    let letter_tok = tokens.next_token()?;
    let letter_span = letter_tok.span;

    let sign = match tokens.peek_token() {
        Some(t) if t.kind == TokenType::MinusSign => {
            let _ = tokens.next_token();
            -1i8
        },
        Some(t) if t.kind == TokenType::PlusSign => {
            let _ = tokens.next_token();
            1i8
        },
        _ => 0i8,
    };
    // Only consume the number token if it's actually a number (recovery: don't swallow next command).
    let is_number = tokens
        .peek_token()
        .map(|t| t.kind == TokenType::Number)
        .unwrap_or(false);
    let num_tok = if is_number { tokens.next_token() } else { None };

    let (num_val, num_span) = match num_tok {
        Some(Token {
            kind: TokenType::Number,
            value,
            span,
        }) => (value, span),
        _ => {
            let mut buf = [0u8; 4];
            let s = letter_c.encode_utf8(&mut buf);
            cmd.diagnostics().emit_unexpected(
                s,
                &[TokenType::Number],
                letter_span,
            );
            return None;
        },
    };

    let value = match literal_value(sign, num_val) {
        Some(v) => v,
        None => {
            cmd.diagnostics().emit_unexpected(
                num_val,
                &[TokenType::Number],
                num_span,
            );
            return None;
        },
    };

    let arg_span = Span::new(
        letter_span.start,
        num_span.end() - letter_span.start,
        letter_span.line,
    );
    cmd.argument(letter_c, value, arg_span);
    Some(arg_span)
}

/// Parse arguments until recovery set. Returns span from command start through last argument.
fn parse_command_arguments<C: CommandVisitor>(
    tokens: &mut Tokens<'_>,
    cmd: &mut C,
    command_start_span: Span,
) -> Span {
    let mut last_span = command_start_span;
    while !at_block_follow(tokens) {
        // Recovery: next command letter ends this command's arguments.
        if matches!(tokens.peek_token(), Some(t) if t.kind == TokenType::Letter)
        {
            if let Some(t) = tokens.peek_token() {
                if t.value.len() == 1 {
                    let c = t.value.chars().next().unwrap_or('\0');
                    if matches!(c, 'G' | 'M' | 'T' | 'N' | 'O') {
                        break;
                    }
                }
            }
        }
        if let Some(arg_span) = parse_argument(tokens, cmd, command_start_span)
        {
            last_span = Span::new(
                command_start_span.start,
                arg_span.end() - command_start_span.start,
                command_start_span.line,
            );
        } else {
            break;
        }
    }
    last_span
}

// ---------- Grammar: command (G / M / T + number) ----------

fn parse_command<B: BlockVisitor>(
    tokens: &mut Tokens<'_>,
    block: &mut B,
    letter_tok: Token<'_>,
    cmd_letter: char,
    _line_start_span: Span,
) -> ControlFlow<()> {
    let number_tok = match tokens.next_token() {
        Some(t) => t,
        None => {
            block.diagnostics().emit_unexpected(
                &letter_tok.value,
                &[TokenType::Eof],
                letter_tok.span,
            );
            return ControlFlow::Continue(());
        },
    };

    let (num_value, num_span) = match number_tok.kind == TokenType::Number {
        true => (number_tok.value, number_tok.span),
        false => {
            block.diagnostics().emit_unexpected(
                &number_tok.value,
                &[TokenType::Number],
                number_tok.span,
            );
            return ControlFlow::Continue(());
        },
    };

    let number = match parse_number(num_value) {
        Some(n) => n,
        None => {
            block.diagnostics().emit_unexpected(
                num_value,
                &[TokenType::Number],
                num_span,
            );
            return ControlFlow::Continue(());
        },
    };

    let cmd_span = Span::new(
        letter_tok.span.start,
        num_span.end() - letter_tok.span.start,
        letter_tok.span.line,
    );

    match cmd_letter {
        'G' => match block.start_general_code(number) {
            ControlFlow::Break(()) => ControlFlow::Break(()),
            ControlFlow::Continue(mut cmd) => {
                let end_span =
                    parse_command_arguments(tokens, &mut cmd, cmd_span);
                cmd.end_command(end_span);
                ControlFlow::Continue(())
            },
        },
        'M' => match block.start_miscellaneous_code(number) {
            ControlFlow::Break(()) => ControlFlow::Break(()),
            ControlFlow::Continue(mut cmd) => {
                let end_span =
                    parse_command_arguments(tokens, &mut cmd, cmd_span);
                cmd.end_command(end_span);
                ControlFlow::Continue(())
            },
        },
        'T' => match block.start_tool_change_code(number) {
            ControlFlow::Break(()) => ControlFlow::Break(()),
            ControlFlow::Continue(mut cmd) => {
                let end_span =
                    parse_command_arguments(tokens, &mut cmd, cmd_span);
                cmd.end_command(end_span);
                ControlFlow::Continue(())
            },
        },
        _ => ControlFlow::Continue(()),
    }
}

// ---------- Grammar: block-level items ----------

/// Line number: N number. Caller has already consumed the N; we only consume the number.
fn parse_line_number<B: BlockVisitor>(
    tokens: &mut Tokens<'_>,
    block: &mut B,
    n_tok: Token<'_>,
    seen_command: bool,
) {
    if seen_command {
        let num_tok = tokens.next_token();
        let span = num_tok
            .as_ref()
            .map(|t| {
                Span::new(
                    n_tok.span.start,
                    t.span.end() - n_tok.span.start,
                    n_tok.span.line,
                )
            })
            .unwrap_or(n_tok.span);
        block.diagnostics().emit_unexpected(
            "N",
            &[TokenType::Letter, TokenType::Number],
            span,
        );
        return;
    }
    match tokens.next_token() {
        Some(Token {
            kind: TokenType::Number,
            value,
            span,
        }) => {
            if let Some(n) = parse_number(value) {
                let n_span = Span::new(
                    n_tok.span.start,
                    span.end() - n_tok.span.start,
                    n_tok.span.line,
                );
                block.line_number(n, n_span);
            }
        },
        _ => {
            block.diagnostics().emit_unexpected(
                "N",
                &[TokenType::Number],
                n_tok.span,
            );
        },
    }
}

/// Program number: O number. Caller has already consumed the O; we only consume the number.
fn parse_program_number<B: BlockVisitor>(
    tokens: &mut Tokens<'_>,
    block: &mut B,
    o_tok: Token<'_>,
    seen_command: bool,
) {
    if seen_command {
        let num_tok = tokens.next_token();
        let span = num_tok
            .as_ref()
            .map(|t| {
                Span::new(
                    o_tok.span.start,
                    t.span.end() - o_tok.span.start,
                    o_tok.span.line,
                )
            })
            .unwrap_or(o_tok.span);
        block.diagnostics().emit_unexpected(
            "O",
            &[TokenType::Letter, TokenType::Number],
            span,
        );
        return;
    }
    match tokens.next_token() {
        Some(Token {
            kind: TokenType::Number,
            value,
            span,
        }) => {
            if let Some(n) = parse_number(value) {
                let o_span = Span::new(
                    o_tok.span.start,
                    span.end() - o_tok.span.start,
                    o_tok.span.line,
                );
                block.program_number(n, o_span);
            }
        },
        _ => {
            block.diagnostics().emit_unexpected(
                "O",
                &[TokenType::Number],
                o_tok.span,
            );
        },
    }
}

/// Comment: ; ... or ( ... ). Consumes one comment token.
#[allow(dead_code)]
fn parse_comment<B: BlockVisitor>(
    tokens: &mut Tokens<'_>,
    block: &mut B,
) -> bool {
    if !matches!(tokens.peek_token(), Some(t) if t.kind == TokenType::Comment) {
        return false;
    }
    let token = tokens.next_token().unwrap();
    block.comment(token.value, token.span);
    true
}

/// One block (line): optional N/O/comments, then zero or more G/M/T commands, until Newline or EOF.
/// Returns (Break if visitor asked to pause, line_span).
fn parse_block_body<'src, B: BlockVisitor>(
    tokens: &mut Tokens<'src>,
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

        // Follow set: end of line.
        if current.kind == TokenType::Newline {
            return (ControlFlow::Continue(()), line_span);
        }

        match current.kind {
            TokenType::Letter => {
                let c = current.value.chars().next().unwrap_or('\0');
                if current.value.len() != 1 {
                    block.diagnostics().emit_unexpected(
                        current.value,
                        &[TokenType::Letter],
                        current.span,
                    );
                    current = match tokens.next_token() {
                        Some(t) => t,
                        None => return (ControlFlow::Continue(()), line_span),
                    };
                    continue;
                }
                match c {
                    'N' => {
                        parse_line_number(tokens, block, current, seen_command);
                        current = match tokens.next_token() {
                            Some(t) => t,
                            None => {
                                return (ControlFlow::Continue(()), line_span);
                            },
                        };
                        continue;
                    },
                    'O' => {
                        parse_program_number(
                            tokens,
                            block,
                            current,
                            seen_command,
                        );
                        current = match tokens.next_token() {
                            Some(t) => t,
                            None => {
                                return (ControlFlow::Continue(()), line_span);
                            },
                        };
                        continue;
                    },
                    'G' | 'M' | 'T' => {
                        seen_command = true;
                        let flow = parse_command(
                            tokens,
                            block,
                            current,
                            c,
                            line_start_span,
                        );
                        if flow.is_break() {
                            return (ControlFlow::Break(()), line_span);
                        }
                        current = match tokens.next_token() {
                            Some(t) => t,
                            None => {
                                return (ControlFlow::Continue(()), line_span);
                            },
                        };
                        continue;
                    },
                    _ => {
                        block.diagnostics().emit_unexpected(
                            current.value,
                            &[TokenType::Letter, TokenType::Comment],
                            current.span,
                        );
                    },
                }
            },
            TokenType::Comment => {
                block.comment(current.value, current.span);
            },
            TokenType::Number => {
                block.diagnostics().emit_unexpected(
                    current.value,
                    &[TokenType::Letter],
                    current.span,
                );
            },
            TokenType::Unknown => {
                block
                    .diagnostics()
                    .emit_unknown_content(current.value, current.span);
            },
            TokenType::Slash | TokenType::MinusSign | TokenType::PlusSign => {
                block.diagnostics().emit_unexpected(
                    current.value,
                    &[TokenType::Letter, TokenType::Comment],
                    current.span,
                );
            },
            TokenType::Newline | TokenType::Eof => {},
        }

        // Progress: consume one token.
        current = match tokens.next_token() {
            Some(t) => t,
            None => return (ControlFlow::Continue(()), line_span),
        };
    }
}

// ---------- Top level ----------

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

    loop {
        // Skip leading newlines so we start at the first token of a line.
        while at(&mut tokens, TokenType::Newline) {
            let _ = tokens.next_token();
        }
        let first = match tokens.next_token() {
            None => break,
            Some(t) => t,
        };
        if first.kind == TokenType::Newline {
            continue;
        }

        let line_start_span = first.span;

        match visitor.start_block() {
            ControlFlow::Break(()) => {
                return_state = Some(tokens.state());
                break;
            },
            ControlFlow::Continue(mut block) => {
                let (flow, line_span) = parse_block_body(
                    &mut tokens,
                    &mut block,
                    first,
                    line_start_span,
                );
                if flow.is_break() {
                    return_state = Some(tokens.state());
                    break;
                }
                block.end_line(line_span);
            },
        }
    }

    return_state.unwrap_or_else(|| tokens.state())
}

impl Default for ParserState {
    fn default() -> Self {
        Self::empty()
    }
}

// ---------- Tests ----------

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
            expected: &[TokenType],
            span: Span,
        ) {
            self.0.push(Event::Unexpected(
                actual.to_string(),
                expected
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
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
                Event::Comment("; hello world".into(), sp(0, 13, 0)),
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
                    sp(0, 26, 0),
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
                    "letter, comment".into(),
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
                    "letter, number".into(),
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

    /// Recovery set: after bad token we continue the line; next valid command is still parsed.
    #[test]
    fn recovery_after_unknown_then_next_command_parsed() {
        let events = parse_and_record("G0 $$ G1 X1");
        assert_eq!(
            events,
            vec![
                Event::LineStarted,
                Event::GeneralCode(Number {
                    major: 0,
                    minor: None
                }),
                Event::UnknownContentError("$$".into(), sp(3, 2, 0)),
                Event::GeneralCode(Number {
                    major: 1,
                    minor: None
                }),
                Event::Argument(
                    'X',
                    crate::ast::Value::Literal(1.0),
                    sp(9, 2, 0)
                ),
            ]
        );
    }

    /// Argument recovery: G/M/T/N/O ends argument list without consuming the next command.
    #[test]
    fn argument_recovery_does_not_swallow_next_command() {
        let events = parse_and_record("G1 X G0 Y1");
        // G1 X (no number) -> error, then we break from arguments; G0 Y1 is a new command.
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::GeneralCode(n) if n.major == 0))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Argument('Y', _, _)))
        );
    }
}
