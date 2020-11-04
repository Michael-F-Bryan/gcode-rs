use crate::Span;

/// Split some text into its constituent tokens.
pub fn tokenize(text: &str) -> impl Iterator<Item = Token<'_>> + '_ {
    Tokenizer::new(text)
}

pub(crate) struct Tokenizer<'input> {
    src: &'input str,
    current_position: usize,
}

impl<'input> Tokenizer<'input> {
    fn new(src: &'input str) -> Self {
        Tokenizer {
            src,
            current_position: 0,
        }
    }

    fn rest(&self) -> &'input str {
        if self.finished() {
            ""
        } else {
            &self.src[self.current_position..]
        }
    }

    fn finished(&self) -> bool { self.current_position >= self.src.len() }

    fn peek(&self) -> Option<TokenType> {
        self.rest().chars().next().map(TokenType::from)
    }

    /// Keep advancing the [`Lexer`] as long as a `predicate` returns `true`,
    /// returning the chomped string, if any.
    fn chomp<F>(&mut self, mut predicate: F) -> Option<&'input str>
    where
        F: FnMut(char) -> bool,
    {
        let start = self.current_position;
        let mut end = start;

        for letter in self.rest().chars() {
            if !predicate(letter) {
                break;
            }
            end += letter.len_utf8();
        }

        if start == end {
            None
        } else {
            self.current_position = end;
            Some(&self.src[start..end])
        }
    }

    fn tokenize_character(&mut self) -> Option<Token<'input>> {
        let c = self.rest().chars().next()?;
        let start = self.current_position;

        if c.is_ascii_alphabetic() {
            self.current_position += 1;
            Some(Token {
                raw: &self.src[start..self.current_position],
                span: Span::new(start, start + 1),
                value: TokenValue::Character(c),
            })
        } else {
            None
        }
    }

    fn tokenize_number(&mut self) -> Option<Token<'input>> { todo!() }

    fn tokenize_newline(&mut self) -> Option<Token<'input>> { todo!() }

    fn tokenize_comment(&mut self) -> Option<Token<'input>> { todo!() }

    fn single_character(
        &mut self,
        value: TokenValue<'input>,
    ) -> Option<Token<'input>> {
        let start = self.current_position;
        let rest = self.rest();
        let next_char = rest.chars().next()?;

        self.current_position += char::len_utf8(next_char);
        let end = self.current_position;

        Some(Token {
            raw: &self.src[start..end],
            span: Span::new(start, end),
            value,
        })
    }
}

impl<'input> Iterator for Tokenizer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        const MSG: &str = "Already checked we're in bounds";

        if self.finished() {
            return None;
        }

        let start = self.current_position;

        while let Some(kind) = self.peek() {
            if kind != TokenType::Unknown && self.current_position != start {
                // we've finished processing some garbage
                let end = self.current_position;
                return Some(Token {
                    raw: &self.src[start..end],
                    span: Span::new(start, end),
                    value: TokenValue::Unknown,
                });
            }

            match kind {
                TokenType::Comment => {
                    return Some(self.tokenize_comment().expect(MSG))
                },
                TokenType::Character => {
                    return Some(self.tokenize_character().expect(MSG))
                },
                TokenType::Number => {
                    return Some(self.tokenize_number().expect(MSG))
                },
                TokenType::Unknown => self.current_position += 1,
                TokenType::Percent => {
                    return Some(
                        self.single_character(TokenValue::Percent).expect(MSG),
                    )
                },
                TokenType::BlockSkip => {
                    return Some(
                        self.single_character(TokenValue::BlockSkip)
                            .expect(MSG),
                    )
                },
                TokenType::Newline => {
                    return Some(self.tokenize_newline().expect(MSG))
                },
            }
        }

        if self.current_position != start {
            // make sure we deal with trailing garbage
            Some(Token {
                raw: &self.src[start..],
                value: TokenValue::Unknown,
                span: Span::new(start, self.current_position),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token<'input> {
    raw: &'input str,
    span: Span,
    value: TokenValue<'input>,
}

impl<'input> Token<'input> {
    pub(crate) fn new(
        raw: &'input str,
        span: Span,
        value: TokenValue<'input>,
    ) -> Self {
        Token { raw, span, value }
    }

    pub const fn raw(&self) -> &'input str { self.raw }

    pub const fn span(&self) -> Span { self.span }

    pub const fn value(&self) -> TokenValue<'input> { self.value }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenValue<'input> {
    /// An ascii character.
    Character(char),
    /// An integer.
    Integer(u32),
    /// A decimal number.
    Float(f64),
    /// A comment.
    Comment(&'input str),
    /// A percent character.
    Percent,
    /// Skip everything from here to the end of the line.
    BlockSkip,
    /// Characters which can't be recognised by the tokenizer.
    Unknown,
    /// A newline character.
    Newline,
}

impl<'input> TokenValue<'input> {
    pub const fn token_type(self) -> TokenType {
        match self {
            TokenValue::Character(_) => TokenType::Character,
            TokenValue::Integer(_) | TokenValue::Float(_) => TokenType::Number,
            TokenValue::Comment(_) => TokenType::Comment,
            TokenValue::Percent => TokenType::Percent,
            TokenValue::BlockSkip => TokenType::BlockSkip,
            TokenValue::Unknown => TokenType::Unknown,
            TokenValue::Newline => TokenType::Newline,
        }
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[repr(C)]
pub enum TokenType {
    /// Characters which can't be recognised by the tokenizer.
    Unknown = 0,
    /// An ascii character.
    Character = 1,
    /// A number.
    Number = 2,
    /// A comment.
    Comment = 3,
    /// A percent character.
    Percent = 4,
    /// Skip everything from here to the end of the line.
    BlockSkip = 5,
    /// A newline character.
    Newline = 6,
}

impl TokenType {
    pub const fn name(self) -> &'static str {
        match self {
            TokenType::Character => "Character",
            TokenType::Number => "Number",
            TokenType::Comment => "Comment",
            TokenType::Percent => "Percent",
            TokenType::BlockSkip => "Block Skip",
            TokenType::Unknown => "Unknown",
            TokenType::Newline => "Newline",
        }
    }
}

impl From<char> for TokenType {
    fn from(c: char) -> TokenType {
        if c.is_alphabetic() {
            return TokenType::Character;
        } else if c.is_ascii_digit() {
            return TokenType::Number;
        }

        match c {
            '.' | '-' | '+' => TokenType::Number,
            '(' | ')' | ';' => TokenType::Comment,
            '%' => TokenType::Percent,
            '/' => TokenType::BlockSkip,
            '\r' | '\n' => TokenType::Newline,
            _ => TokenType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokenize_test {
        ($name:ident, $src:expr, $should_be:expr) => {
            #[test]
            fn $name() {
                let src = $src;
                let mut tokenizer = Tokenizer::new(src);

                let got = tokenizer.next().unwrap();

                assert_eq!(got.raw, src);
                assert_eq!(got.span, Span::new(0, src.len()));
                assert_eq!(got.value, $should_be);

                assert!(tokenizer.finished());
                assert!(tokenizer.rest().is_empty());
            }
        };
    }

    tokenize_test!(single_uppercase_character, "G", TokenValue::Character('G'));
    tokenize_test!(single_lowercase_character, "g", TokenValue::Character('g'));
    tokenize_test!(random_garbage, "$", TokenValue::Unknown);
    tokenize_test!(percent, "%", TokenValue::Percent);
    tokenize_test!(block_skip, "/", TokenValue::BlockSkip);
}
