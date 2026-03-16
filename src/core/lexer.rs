use crate::core::{ParserState, Span};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum TokenType {
    Letter,
    Number,
    Comment,
    /// A `/` is used to indicate a deleted block.
    Slash,
    MinusSign,
    PlusSign,
    Newline,
    Unknown,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Token<'input> {
    pub(crate) kind: TokenType,
    pub(crate) value: &'input str,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Tokens<'src> {
    src: &'src str,
    current_index: usize,
    current_line: usize,
}

impl<'src> Tokens<'src> {
    pub(crate) fn new(
        src: &'src str,
        current_index: usize,
        current_line: usize,
    ) -> Self {
        Self {
            src,
            current_index,
            current_line,
        }
    }

    pub(crate) fn state(self) -> ParserState {
        let Self {
            current_index,
            current_line,
            ..
        } = self;
        ParserState::new(current_index, current_line)
    }

    /// Current parse position (for use while the iterator is borrowed, e.g. by a peekable).
    pub(crate) fn state_ref(&self) -> ParserState {
        ParserState::new(self.current_index, self.current_line)
    }

    /// Returns the next character without advancing. Uses UTF-8 decoding.
    fn peek(&self) -> Option<char> {
        self.src[self.current_index..].chars().next()
    }

    /// Consumes the next character, bumping `current_index` and `current_line` (when the char is `\n`).
    /// Returns the character consumed, or `None` if at end. This is the only place that updates
    /// `current_index` and `current_line`.
    fn advance_char(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current_index += c.len_utf8();
        if c == '\n' {
            self.current_line += 1;
        }
        Some(c)
    }

    /// Consumes characters while `pred(c)` returns true, then returns the slice and span.
    /// Uses `advance_char()` internally, so line is updated for newlines.
    fn take_while<P>(&mut self, mut pred: P) -> (&'src str, Span)
    where
        P: FnMut(char) -> bool,
    {
        let start = self.current_index;
        let line = self.current_line;
        while let Some(c) = self.peek() {
            if !pred(c) {
                break;
            }
            let _ = self.advance_char();
        }
        let value = &self.src[start..self.current_index];
        let span = Span::new(start, value.len(), line);
        (value, span)
    }

    fn skip_whitespace(&mut self) {
        let _ = self.take_while(|c| c == ' ' || c == '\t');
    }

    fn scan_newline(&mut self) -> Token<'src> {
        let start = self.current_index;
        let line = self.current_line;
        let c = self.advance_char();
        assert_eq!(c, Some('\n'), "invariant: peek was newline");
        Token {
            kind: TokenType::Newline,
            value: &self.src[start..self.current_index],
            span: Span::new(start, self.current_index - start, line),
        }
    }

    fn scan_letter(&mut self) -> Token<'src> {
        let start = self.current_index;
        let line = self.current_line;
        let _ = self.advance_char();
        Token {
            kind: TokenType::Letter,
            value: &self.src[start..self.current_index],
            span: Span::new(start, self.current_index - start, line),
        }
    }

    fn scan_single_char(&mut self, kind: TokenType) -> Token<'src> {
        let start = self.current_index;
        let line = self.current_line;
        let _ = self.advance_char();
        Token {
            kind,
            value: &self.src[start..self.current_index],
            span: Span::new(start, self.current_index - start, line),
        }
    }

    fn scan_semicolon_comment(&mut self) -> Token<'src> {
        let (value, span) = self.take_while(|c| c != '\n');
        Token {
            kind: TokenType::Comment,
            value,
            span,
        }
    }

    fn scan_paren_comment(&mut self) -> Token<'src> {
        let start = self.current_index;
        let line = self.current_line;
        let mut depth = 1u32;
        let _ = self.advance_char(); // consume '('
        while depth > 0 {
            match self.advance_char() {
                Some('(') => depth += 1,
                Some(')') => depth -= 1,
                Some(_) => {},
                None => break,
            }
        }
        Token {
            kind: TokenType::Comment,
            value: &self.src[start..self.current_index],
            span: Span::new(start, self.current_index - start, line),
        }
    }

    fn scan_number(&mut self) -> Token<'src> {
        let start = self.current_index;
        let line = self.current_line;
        if self.peek() == Some('.') {
            let _ = self.advance_char();
        }
        let _ = self.take_while(|c| c.is_ascii_digit());
        if self.peek() == Some('.') {
            let _ = self.advance_char();
            let _ = self.take_while(|c| c.is_ascii_digit());
        }
        Token {
            kind: TokenType::Number,
            value: &self.src[start..self.current_index],
            span: Span::new(start, self.current_index - start, line),
        }
    }

    fn scan_unknown(&mut self) -> Token<'src> {
        let (value, span) = self.take_while(|c| {
            !matches!(c, ' ' | '\t' | '\n' | ';' | '(' | '-' | '+')
                && !c.is_ascii_alphanumeric()
        });
        Token {
            kind: TokenType::Unknown,
            value,
            span,
        }
    }
}

impl<'src> Iterator for Tokens<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let token = match self.peek()? {
            '\n' => self.scan_newline(),
            c if c.is_ascii_alphabetic() => self.scan_letter(),
            '-' => self.scan_single_char(TokenType::MinusSign),
            '+' => self.scan_single_char(TokenType::PlusSign),
            '/' => self.scan_single_char(TokenType::Slash),
            ';' => self.scan_semicolon_comment(),
            '(' => self.scan_paren_comment(),
            c if c.is_ascii_digit() || c == '.' => self.scan_number(),
            _ => self.scan_unknown(),
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Span;

    use super::*;

    impl<'src> Tokens<'src> {
        pub(crate) fn from_start(src: &'src str) -> Self {
            Self::new(src, 0, 0)
        }
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        let tokens: Vec<_> = Tokens::from_start("").collect();
        assert_eq!(tokens, []);
    }

    #[test]
    fn whitespace_only_yields_no_tokens() {
        let tokens: Vec<_> = Tokens::from_start("   ").collect();
        assert_eq!(tokens, []);
    }

    #[test]
    fn whitespace_with_tabs_yields_no_tokens() {
        let tokens: Vec<_> = Tokens::from_start(" \t ").collect();
        assert_eq!(tokens, []);
    }

    #[test]
    fn newline_yields_newline_token() {
        let src = "\n";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Newline,
                value: "\n",
                span: Span::new(0, 1, 0),
            }]
        );
    }

    #[test]
    fn two_newlines_yield_two_newline_tokens() {
        let src = "\n\n";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Newline,
                    value: "\n",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Newline,
                    value: "\n",
                    span: Span::new(1, 1, 1),
                },
            ]
        );
    }

    #[test]
    fn single_letter_yields_letter_token() {
        let src = "G";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Letter,
                value: "G",
                span: Span::new(0, 1, 0),
            }]
        );
    }

    #[test]
    fn letter_then_number_span_correctly() {
        let src = "G90";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Letter,
                    value: "G",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "90",
                    span: Span::new(1, 2, 0),
                },
            ]
        );
    }

    #[test]
    fn number_integer() {
        let src = "42";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Number,
                value: "42",
                span: Span::new(0, 2, 0),
            }]
        );
    }

    #[test]
    fn number_zero() {
        let src = "0";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Number,
                value: "0",
                span: Span::new(0, 1, 0),
            }]
        );
    }

    #[test]
    fn number_float() {
        let src = "3.14";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Number,
                value: "3.14",
                span: Span::new(0, 4, 0),
            }]
        );
    }

    #[test]
    fn number_negative() {
        let src = "-1";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::MinusSign,
                    value: "-",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "1",
                    span: Span::new(1, 1, 0),
                }
            ]
        );
    }

    #[test]
    fn number_positive() {
        let src = "+1";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::PlusSign,
                    value: "+",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "1",
                    span: Span::new(1, 1, 0),
                }
            ]
        );
    }

    #[test]
    fn number_with_trailing_dot_matches_old_lexer() {
        let src = "3.14.56";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Number,
                    value: "3.14",
                    span: Span::new(0, 4, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: ".56",
                    span: Span::new(4, 3, 0),
                },
            ]
        );
    }

    #[test]
    fn semicolon_comment_includes_semicolon_and_rest_of_line() {
        let src = "; this is a comment\n";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Comment,
                    value: "; this is a comment",
                    span: Span::new(0, 19, 0),
                },
                Token {
                    kind: TokenType::Newline,
                    value: "\n",
                    span: Span::new(19, 1, 0),
                },
            ]
        );
    }

    #[test]
    fn parens_comment_block() {
        let src = "( block )";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Comment,
                value: "( block )",
                span: Span::new(0, 9, 0),
            }]
        );
    }

    #[test]
    fn unclosed_paren_comment_treated_as_unknown_or_comment_to_eol() {
        let src = "( no close";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        let expected_comment = [Token {
            kind: TokenType::Comment,
            value: "( no close",
            span: Span::new(0, 10, 0),
        }];
        let expected_unknown = [Token {
            kind: TokenType::Unknown,
            value: "( no close",
            span: Span::new(0, 10, 0),
        }];
        assert!(
            tokens == expected_comment.as_slice()
                || tokens == expected_unknown.as_slice(),
            "unclosed paren yields Comment or Unknown"
        );
    }

    #[test]
    fn unknown_single_run() {
        let src = "$#@";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Unknown,
                value: "$#@",
                span: Span::new(0, 3, 0),
            }]
        );
    }

    #[test]
    fn unknown_then_whitespace_then_letter() {
        let src = "$ X";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Unknown,
                    value: "$",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "X",
                    span: Span::new(2, 1, 0),
                },
            ]
        );
    }

    #[test]
    fn mixed_g1_x100() {
        let src = "G1  X100"; // two spaces so X at 4, "100" at 5..8
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Letter,
                    value: "G",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "1",
                    span: Span::new(1, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "X",
                    span: Span::new(4, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "100",
                    span: Span::new(5, 3, 0),
                },
            ]
        );
    }

    #[test]
    fn mixed_n10_g0_newline_second_line_has_line_one() {
        let src = "N10 G0\n";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Letter,
                    value: "N",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "10",
                    span: Span::new(1, 2, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "G",
                    span: Span::new(4, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "0",
                    span: Span::new(5, 1, 0),
                },
                Token {
                    kind: TokenType::Newline,
                    value: "\n",
                    span: Span::new(6, 1, 0),
                },
            ]
        );
    }

    #[test]
    fn span_consistency_each_token_value_matches_slice() {
        let src = "G1 X100\n; comment";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        for token in &tokens {
            let slice = &src[token.span.start..token.span.end()];
            assert_eq!(
                token.value, slice,
                "token value must equal src[span.start..span.end]"
            );
        }
    }

    #[test]
    fn multiple_letters_is_multiple_tokens() {
        let src = "GTM";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenType::Letter,
                    value: "G",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "T",
                    span: Span::new(1, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "M",
                    span: Span::new(2, 1, 0),
                },
            ],
        );
    }

    #[test]
    fn current_index_is_end_when_finished() {
        let src = "G1 X100";
        let mut tokens = Tokens::from_start(src);
        let _: Vec<_> = tokens.by_ref().collect();
        assert_eq!(tokens.current_index, src.len());
    }

    #[test]
    fn slash_is_deleted_block() {
        let src = "/G1\n";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        println!("{:?}", tokens);
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Slash,
                    value: "/",
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: "G",
                    span: Span::new(1, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: "1",
                    span: Span::new(2, 1, 0),
                },
                Token {
                    kind: TokenType::Newline,
                    value: "\n",
                    span: Span::new(3, 1, 0),
                }
            ],
        );
    }
}
