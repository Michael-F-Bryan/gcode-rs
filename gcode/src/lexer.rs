use types::Span;

struct Lexer<'input> {
    src: &'input str,
    current_index: usize,
    current_line: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(src: &'input str) -> Lexer<'input> {
        Lexer {
            src,
            current_index: 0,
            current_line: 0,
        }
    }

    fn step(&mut self) -> Option<Token<'input>> {
        let next = self.peek()?;

        match next {
            '\n' => Some(self.tokenize_newline()),
            '(' | ';' => Some(self.tokenize_comment()),
            '%' => Some(self.tokenize_percent()),
            '/' => Some(self.tokenize_forward_slash()),
            other if other.is_ascii_alphabetic() => {
                Some(self.tokenize_letter())
            }
            _ => unimplemented!(),
        }
    }

    fn take_while<P>(&mut self, mut predicate: P) -> &'input str
    where
        P: FnMut(char) -> bool,
    {
        let start = self.current_index;

        while let Some(c) = self.peek() {
            if predicate(c) {
                self.advance();
            } else {
                break;
            }
        }

        &self.src[start..self.current_index]
    }

    fn tokenize_forward_slash(&mut self) -> Token<'input> {
        let slash = self.advance().unwrap();
        debug_assert!(slash == '/');

        Token::ForwardSlash
    }

    fn tokenize_percent(&mut self) -> Token<'input> {
        let percent = self.advance().unwrap();
        debug_assert!(percent == '%');

        let comment = self.take_while(|c| c != '\n');

        // skip past the newline
        let _ = self.advance();

        if comment.is_empty() {
            Token::Percent(None)
        } else {
            Token::Percent(Some(comment))
        }
    }

    fn tokenize_comment(&mut self) -> Token<'input> {
        let start = self.current_index;
        let comment_char = self.advance().unwrap();

        let end_of_comment = match comment_char {
            '(' => ')',
            ';' => '\n',
            _ => unreachable!(),
        };

        // skip the comment body
        let _ = self.take_while(|c| c != end_of_comment);
        // step past the end-of-comment character
        let _ = self.advance();

        Token::Comment(&self.src[start..self.current_index])
    }

    fn tokenize_letter(&mut self) -> Token<'input> {
        let c = self.advance().unwrap();
        debug_assert!(c.is_ascii_alphabetic());

        Token::Letter(c)
    }

    fn tokenize_newline(&mut self) -> Token<'input> {
        let c = self.advance().unwrap();
        debug_assert!(c == '\n');

        Token::Newline
    }

    fn advance(&mut self) -> Option<char> {
        let next = self.peek();

        if let Some(c) = next {
            self.current_index += c.len_utf8();
            if c == '\n' {
                self.current_line += 1;
            }
        }

        next
    }

    fn peek(&self) -> Option<char> {
        self.src[self.current_index..].chars().next()
    }

    fn here(&self) -> Span {
        Span {
            start: self.current_index,
            end: self.current_index,
            source_line: self.current_line,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = (Token<'input>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let mut span = self.here();

        let tok = self.step()?;
        span.end = self.current_index;

        Some((tok, span))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token<'input> {
    Letter(char),
    Number(f32),
    Comment(&'input str),
    Newline,
    ForwardSlash,
    /// A `%` delimiter with optional comment.
    Percent(Option<&'input str>),
}

impl<'input> From<char> for Token<'input> {
    fn from(other: char) -> Self {
        Token::Letter(other)
    }
}

impl<'input> From<f32> for Token<'input> {
    fn from(other: f32) -> Self {
        Token::Number(other)
    }
}

impl<'input> From<i32> for Token<'input> {
    fn from(other: i32) -> Self {
        Token::Number(other as f32)
    }
}

impl<'input> From<&'input str> for Token<'input> {
    fn from(other: &'input str) -> Self {
        Token::Comment(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! lexer_test {
        ($name:ident, $src:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let src = $src;
                let should_be = Token::from($should_be);
                let mut lexy = Lexer::new(src);

                let (token, span) = lexy.next().unwrap();

                assert_eq!(token, should_be);
                assert_eq!(
                    span,
                    Span {
                        start: 0,
                        end: src.len(),
                        source_line: 0,
                    }
                );
                assert_eq!(lexy.current_index, src.len());
            }
        };
    }

    lexer_test!(lex_a_letter, "W" => Token::Letter('W'));
    lexer_test!(lex_a_lowercase_letter, "g" => 'g');
    lexer_test!(lex_comment_in_parens, "(this is a comment)" => "(this is a comment)");
    lexer_test!(lex_bare_percent, "%" => Token::Percent(None));
    lexer_test!(lex_bare_percent_with_newline, "%\n" => Token::Percent(None));
    lexer_test!(lex_percent_with_comment, "% This is a comment\n" => Token::Percent(Some(" This is a comment")));
    lexer_test!(lex_a_forward_slash, "/" => Token::ForwardSlash);

    #[test]
    fn recognise_a_newline() {
        let src = "\n";
        let should_be = Token::Newline;
        let mut lexy = Lexer::new(src);

        let (token, span) = lexy.next().unwrap();

        assert_eq!(token, should_be);
        assert_eq!(
            span,
            Span {
                start: 0,
                end: src.len(),
                source_line: 0,
            }
        );
        assert_eq!(lexy.current_line, 1);
    }
}
