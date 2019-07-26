use crate::Span;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum TokenType {
    Letter,
    Number,
    Comment,
    Unknown,
}

impl From<char> for TokenType {
    fn from(c: char) -> TokenType {
        if c.is_ascii_alphabetic() {
            TokenType::Letter
        } else if c.is_ascii_digit() || c == '.' {
            TokenType::Number
        } else if c == '(' || c == ';' || c == ')' {
            TokenType::Comment
        } else {
            TokenType::Unknown
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Token<'input> {
    pub kind: TokenType,
    pub value: &'input str,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Lexer<'input> {
    current_position: usize,
    current_line: usize,
    src: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(src: &'input str) -> Self {
        Lexer {
            current_position: 0,
            current_line: 0,
            src,
        }
    }

    /// Keep advancing the [`Lexer`] as long as a `predicate` returns `true`,
    /// returning the chomped string, if any.
    fn chomp<F>(&mut self, mut predicate: F) -> Option<&'input str>
    where
        F: FnMut(char) -> bool,
    {
        let start = self.current_position;
        let mut end = start;
        let mut line_endings = 0;

        for letter in self.rest().chars() {
            if !predicate(letter) {
                break;
            }
            if letter == '\n' {
                line_endings += 1;
            }
            end += letter.len_utf8();
        }

        if start == end {
            None
        } else {
            self.current_position = end;
            self.current_line += line_endings;
            Some(&self.src[start..end])
        }
    }

    fn rest(&self) -> &'input str {
        if self.finished() {
            ""
        } else {
            &self.src[self.current_position..]
        }
    }

    fn skip_whitespace(&mut self) {
        let _ = self.chomp(char::is_whitespace);
    }

    fn tokenize_comment(&mut self) -> Option<Token<'input>> {
        let start = self.current_position;
        let line = self.current_line;

        if self.rest().starts_with(';') {
            // the comment is every character from ';' to '\n' or EOF
            let comment = self.chomp(|c| c != '\n').unwrap_or("");
            let end = self.current_position;

            Some(Token {
                kind: TokenType::Comment,
                value: comment,
                span: Span { start, end, line },
            })
        } else if self.rest().starts_with('(') {
            // skip past the comment body
            let _ = self.chomp(|c| c != '\n' && c != ')');

            // at this point, it's guaranteed that the next character is '\n',
            // ')' or EOF
            let kind = self.peek().unwrap_or(TokenType::Unknown);

            if kind == TokenType::Comment {
                // we need to consume the closing paren
                self.current_position += 1;
            }

            let end = self.current_position;
            let value = &self.src[start..end];

            Some(Token {
                kind,
                value,
                span: Span { start, end, line },
            })
        } else {
            None
        }
    }

    fn tokenize_letter(&mut self) -> Option<Token<'input>> {
        let c = self.rest().chars().next()?;
        let start = self.current_position;

        if c.is_ascii_alphabetic() {
            self.current_position += 1;
            Some(Token {
                kind: TokenType::Letter,
                value: &self.src[start..=start],
                span: Span {
                    start,
                    end: start + 1,
                    line: self.current_line,
                },
            })
        } else {
            None
        }
    }

    fn tokenize_number(&mut self) -> Option<Token<'input>> {
        let mut decimal_seen = false;
        let start = self.current_position;
        let line = self.current_line;

        let value = self.chomp(|c| {
            if c.is_ascii_digit() {
                true
            } else if c == '.' && !decimal_seen {
                decimal_seen = true;
                true
            } else {
                false
            }
        })?;

        Some(Token {
            kind: TokenType::Number,
            value,
            span: Span {
                start,
                line,
                end: self.current_position,
            },
        })
    }

    fn finished(&self) -> bool {
        self.current_position >= self.src.len()
    }

    fn peek(&self) -> Option<TokenType> {
        self.rest().chars().next().map(TokenType::from)
    }
}

impl<'input> From<&'input str> for Lexer<'input> {
    fn from(other: &'input str) -> Lexer<'input> {
        Lexer::new(other)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        const MSG: &str = "This should be unreachable, we've already done a bounds check";
        self.skip_whitespace();

        let start = self.current_position;
        let line = self.current_line;

        while let Some(kind) = self.peek() {
            match kind {
                TokenType::Comment => return Some(self.tokenize_comment().expect(MSG)),
                TokenType::Letter => return Some(self.tokenize_letter().expect(MSG)),
                TokenType::Number => return Some(self.tokenize_number().expect(MSG)),
                TokenType::Unknown => self.current_position += 1,
            }
        }

        if self.current_position != start {
            // make sure we deal with trailing garbage
            Some(Token {
                kind: TokenType::Unknown,
                value: &self.src[start..],
                span: Span {
                    start,
                    end: self.current_position,
                    line,
                },
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_while_works_as_expected() {
        let mut lexer = Lexer::new("12345abcd");

        let got = lexer.chomp(|c| c.is_digit(10));

        assert_eq!(got, Some("12345"));
        assert_eq!(lexer.current_position, 5);
        assert_eq!(lexer.rest(), "abcd");
    }

    #[test]
    fn skip_whitespace() {
        let mut lexer = Lexer::new("  \n\r\t  ");

        lexer.skip_whitespace();

        assert_eq!(lexer.current_position, lexer.src.len());
        assert_eq!(lexer.current_line, 1);
    }

    #[test]
    fn tokenize_a_semicolon_comment() {
        let mut lexer = Lexer::new("; this is a comment\nbut this is not");
        let newline = lexer.src.find('\n').unwrap();

        let got = lexer.next().unwrap();

        assert_eq!(got.value, "; this is a comment");
        assert_eq!(got.kind, TokenType::Comment);
        assert_eq!(
            got.span,
            Span {
                start: 0,
                end: newline,
                line: 0
            }
        );
        assert_eq!(lexer.current_position, newline);
    }

    #[test]
    fn tokenize_a_parens_comment() {
        let mut lexer = Lexer::new("( this is a comment) but this is not");
        let comment = "( this is a comment)";

        let got = lexer.next().unwrap();

        assert_eq!(got.value, comment);
        assert_eq!(got.kind, TokenType::Comment);
        assert_eq!(
            got.span,
            Span {
                start: 0,
                end: comment.len(),
                line: 0
            }
        );
        assert_eq!(lexer.current_position, comment.len());
    }

    #[test]
    fn unclosed_parens_are_garbage() {
        let mut lexer = Lexer::new("( missing a closing paren");

        let got = lexer.next().unwrap();

        assert_eq!(got.value, lexer.src);
        assert_eq!(got.kind, TokenType::Unknown);
        assert_eq!(got.span.end, lexer.src.len());
        assert_eq!(lexer.current_position, lexer.src.len());
    }

    #[test]
    fn tokenize_a_letter() {
        let mut lexer = Lexer::new("asd\nf");

        let got = lexer.next().unwrap();

        assert_eq!(got.value, "a");
        assert_eq!(got.kind, TokenType::Letter);
        assert_eq!(got.span.end, 1);
        assert_eq!(lexer.current_position, 1);
    }

    #[test]
    fn normal_number() {
        let mut lexer = Lexer::new("3.14.56\nf");

        let got = lexer.next().unwrap();

        assert_eq!(got.value, "3.14");
        assert_eq!(got.kind, TokenType::Number);
        assert_eq!(got.span.end, 4);
        assert_eq!(lexer.current_position, 4);
    }
}
