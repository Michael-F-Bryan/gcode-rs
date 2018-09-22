use types::Span;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token<'input> {
    Letter(char),
    Number(f32),
    Comment(&'input str),
    Newline,
    Percent,
}

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

    fn parse_letter(&mut self) -> Option<Token<'input>> {
        let c = self.peek()?;

        if c.is_ascii_alphabetic() {
            self.advance();
            Some(Token::Letter(c))
        } else {
            None
        }
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

        let letter = self.parse_letter()?;
        span.end = self.current_index;

        Some((letter, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_a_letter() {
        let src = "W";

        let (token, span) = Lexer::new(src).next().unwrap();

        assert_eq!(token, Token::Letter('W'));
        assert_eq!(
            span,
            Span {
                start: 0,
                end: 1,
                source_line: 0
            }
        );
    }
}
