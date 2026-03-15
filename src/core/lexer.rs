use crate::core::{ParserState, Span};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum TokenType {
    Letter,
    Number,
    Comment,
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
    pub(crate) fn from_start(src: &'src str) -> Self {
        Self::new(src, 0, 0)
    }

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
}

impl<'src> Iterator for Tokens<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Span;

    use super::*;

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
                    value: &src[0..1],
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[1..3],
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
                    value: "+1",
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
                    value: &src[0..4],
                    span: Span::new(0, 4, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[4..8],
                    span: Span::new(4, 4, 0),
                },
            ]
        );
    }

    #[test]
    fn semicolon_comment_includes_semicolon_and_rest_of_line() {
        let src = "; this is a comment\n";
        let newline_pos = src.find('\n').unwrap();
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [Token {
                kind: TokenType::Comment,
                value: &src[0..newline_pos],
                span: Span::new(0, newline_pos, 0),
            }]
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
                value: &src[0..src.len()],
                span: Span::new(0, src.len(), 0),
            }]
        );
    }

    #[test]
    fn unclosed_paren_comment_treated_as_unknown_or_comment_to_eol() {
        let src = "( no close";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        let expected_comment = [Token {
            kind: TokenType::Comment,
            value: &src[..],
            span: Span::new(0, src.len(), 0),
        }];
        let expected_unknown = [Token {
            kind: TokenType::Unknown,
            value: &src[..],
            span: Span::new(0, src.len(), 0),
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
                    value: &src[0..1],
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: &src[2..3],
                    span: Span::new(2, 1, 0),
                },
            ]
        );
    }

    #[test]
    fn mixed_g1_x100() {
        let src = "G1 X100";
        let tokens: Vec<_> = Tokens::from_start(src).collect();
        assert_eq!(
            tokens,
            [
                Token {
                    kind: TokenType::Letter,
                    value: &src[0..1],
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[1..2],
                    span: Span::new(1, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: &src[4..5],
                    span: Span::new(4, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[5..8],
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
                    value: &src[0..1],
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[1..3],
                    span: Span::new(1, 2, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: &src[4..5],
                    span: Span::new(4, 1, 0),
                },
                Token {
                    kind: TokenType::Number,
                    value: &src[5..6],
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
                    value: &src[0..1],
                    span: Span::new(0, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: &src[1..2],
                    span: Span::new(1, 1, 0),
                },
                Token {
                    kind: TokenType::Letter,
                    value: &src[2..3],
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
}
