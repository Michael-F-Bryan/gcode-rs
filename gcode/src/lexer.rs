use arrayvec::ArrayString;
use types::Span;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Lexer<'input> {
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

    pub fn src(&self) -> &'input str {
        self.src
    }

    fn step(&mut self) -> Option<Token<'input>> {
        let next = self.peek()?;

        match next {
            '\n' => Some(self.tokenize_newline()),
            '(' | ';' => Some(self.tokenize_comment()),
            '%' => Some(self.tokenize_percent()),
            '/' => Some(self.tokenize_forward_slash()),
            '.' => Some(self.tokenize_number()),
            other if other.is_numeric() => Some(self.tokenize_number()),
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

    fn tokenize_number(&mut self) -> Token<'input> {
        // gcode numbers are funny. They can sometimes contain internal
        // whitespace, so we can't directly use `f32::from_str()`. Instead we
        // copy to a temporary buffer, read the number, then try to parse that.
        let mut buffer: ArrayString<[u8; 32]> = ArrayString::new();
        let mut seen_decimal = false;
        let mut input_is_malformed = false;
        let start = self.current_index;

        while let Some(next) = self.peek() {
            if next == '\n' || next == '\r' {
                break;
            }

            if next == '.' {
                if seen_decimal {
                    break;
                } else {
                    seen_decimal = true;
                }
            }

            if next != '.' && !next.is_numeric() && !next.is_whitespace() {
                break;
            }

            if !next.is_whitespace()
                && !input_is_malformed
                && buffer.try_push(next).is_err()
            {
                // Pushing any more characters would overflow our buffer.
                // You can't really parse a 32-digit number without loss of
                // precision anyway, so from here on we're going to pretend
                // the whole thing is malformed and garbage.
                input_is_malformed = true;
            }

            let _ = self.advance();
        }

        if input_is_malformed {
            Token::GarbageNumber(&self.src[start..self.current_index])
        } else {
            Token::Number(buffer.parse().expect("Parse should never fail"))
        }
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
        let mut end = self.current_index;

        // we want to include the closing paren, but ignore a trailing newline
        if end_of_comment == ')' {
            // step past the end-of-comment character
            let _ = self.advance();
            end = self.current_index;
        }

        Token::Comment(&self.src[start..end])
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

    fn rest(&self) -> &'input str {
        &self.src[self.current_index..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
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
pub(crate) enum Token<'input> {
    Letter(char),
    Number(f32),
    Comment(&'input str),
    Newline,
    ForwardSlash,
    /// A `%` delimiter with optional comment.
    Percent(Option<&'input str>),
    /// A stupidly long decimal number was encountered. It'd normally overflow
    /// and break stuff if we tried to parse it, so pass it through to the
    /// parser as an erroneous variant.
    GarbageNumber(&'input str),
}

impl<'input> Token<'input> {
    pub const LETTER: &'static str = "letter";
    pub const NUMBER: &'static str = "number";
    pub const COMMENT: &'static str = "comment";
    pub const NEWLINE: &'static str = "newline";
    pub const FORWARD_SLASH: &'static str = "forward-slash";
    pub const PERCENT: &'static str = "percent";
    pub const PERCENT_WITH_COMMENT: &'static str = "percent-with-comment";
    pub const GARBAGE: &'static str = "garbage";

    pub fn is_err(&self) -> bool {
        match *self {
            Token::GarbageNumber(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Token::Number(_) => true,
            _ => false,
        }
    }

    pub fn kind(&self) -> &'static str {
        match *self {
            Token::Letter(_) => Self::LETTER,
            Token::Number(_) => Self::NUMBER,
            Token::Comment(_) => Self::COMMENT,
            Token::Newline => Self::NEWLINE,
            Token::ForwardSlash => Self::FORWARD_SLASH,
            Token::Percent(None) => Self::PERCENT,
            Token::Percent(Some(_)) => Self::PERCENT_WITH_COMMENT,
            Token::GarbageNumber(_) => Self::GARBAGE,
        }
    }
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
            lexer_test!{
                $name, $src => $should_be;
                |lexy, span, src| {
                    assert_eq!(span, Span { start: 0, end: src.len(), source_line: 0 });
                    assert_eq!(lexy.current_index, src.len());
                }
            }
        };
        (ignore_span $name:ident, $src:expr => $should_be:expr) => {
            lexer_test!($name, $src => $should_be; |_lexy, _span, _src| {});
        };
        ($name:ident, $src:expr => $should_be:expr;
         |$lexer_name:ident, $span_name:ident, $src_name:ident| $span_check:expr) => {
            #[test]
            fn $name() {
                let $src_name = $src;
                let should_be = Token::from($should_be);
                let mut $lexer_name = Lexer::new($src_name);

                let (token, $span_name) = $lexer_name.next().unwrap();

                assert_eq!(token, should_be);
                $span_check;
            }
        };
    }

    lexer_test!(lex_a_letter, "W" => Token::Letter('W'));
    lexer_test!(lex_a_lowercase_letter, "g" => 'g');
    lexer_test!(lex_comment_in_parens, "(this is a comment)" => "(this is a comment)");
    lexer_test!(ignore_span lex_newline_comment, "; this is a comment\n" =>"; this is a comment");
    lexer_test!(lex_bare_percent, "%" => Token::Percent(None));
    lexer_test!(lex_bare_percent_with_newline, "%\n" => Token::Percent(None));
    lexer_test!(lex_percent_with_comment, "% This is a comment\n" => Token::Percent(Some(" This is a comment")));
    lexer_test!(lex_a_forward_slash, "/" => Token::ForwardSlash);
    lexer_test!(integer, "42" => 42);
    lexer_test!(decimal, "1.23" => 1.23);
    lexer_test!(integer_with_space, "1 23" => 123);
    lexer_test!(funky_spaces, "1 23. 4 5" => 123.45);
    lexer_test!(ignore_long_numbers_as_malformed, "1234567890 1234567890 1234567890 1234567890" =>
                Token::GarbageNumber("1234567890 1234567890 1234567890 1234567890"));
    lexer_test!(no_leading_zero, ".5" => 0.5);

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
