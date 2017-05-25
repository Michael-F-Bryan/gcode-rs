//! A module for turning raw gcode into tokens to be processed by the parser.

use core::iter::Peekable;

use errors::*;
use helpers::*;


/// A zero-allocation tokenizer.
///
/// # Examples
///
/// ```rust
/// use gcode::lexer::Tokenizer;
/// let src = "N40 G90 X1.0";
/// let tokens: Vec<_> = Tokenizer::new(src.chars()).collect();
/// ```
pub struct Tokenizer<I>
    where I: Iterator<Item = char>
{
    src: Peekable<I>,
    line: usize,
    column: usize,
}


impl<I> Tokenizer<I>
    where I: Iterator<Item = char>
{
    /// Create a new `Tokenizer` from some `char` iterator.
    pub fn new(src: I) -> Self {
        Tokenizer {
            src: src.peekable(),
            line: 0,
            column: 0,
        }
    }

    fn next_token(&mut self) -> Option<Result<Token>> {
        while let Some(peek) = self.next_char() {
            if peek.is_whitespace() {
                continue;
            }

            let span = Span {
                line: self.line,
                column: self.column,
            };

            let tok = match peek {
                d if d.is_digit(10) || d == '.' => self.tokenize_number(d, span),
                a if a.is_alphabetic() => self.tokenize_alpha(a, span),

                ';' => {
                    self.to_end_of_line();
                    continue;
                }
                '(' => {
                    self.skip_comment();
                    continue;
                }

                '%' => {
                    Ok(Token {
                           kind: TokenKind::Percent,
                           span: span,
                       })
                }
                '-' => {
                    Ok(Token {
                           kind: TokenKind::Minus,
                           span: span,
                       })
                }

                other => Err(Error::UnknownToken(other, span)),
            };

            return Some(tok);
        }

        None
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.src.next();

        if let Some(n) = next {
            self.column += 1;
            if n == '\n' {
                self.line += 1;
                self.column = 0;
            }
        }

        next
    }

    fn tokenize_number(&mut self, first: char, span: Span) -> Result<Token> {
        // TODO: Make clean... pls
        let (integer_part, _) = self.tokenize_integer(first);

        match self.src.peek() {
            Some(&'.') => {}
            _ => {
                let kind = TokenKind::Integer(integer_part);
                return Ok(Token { kind, span });
            }
        }

        let _ = self.next_char();

        let kind = match self.src.peek().cloned() {
            Some(d) if d.is_digit(10) => {
                let next = self.next_char().unwrap();
                let (fractional_part, length) = self.tokenize_integer(next);

                let number = float_from_integers(integer_part, fractional_part, length);
                TokenKind::Number(number)
            }
            _ => TokenKind::Number(integer_part as f32),
        };

        Ok(Token { kind, span })
    }

    fn tokenize_integer(&mut self, first: char) -> (u32, u32) {
        // We've already established that `first` is 0..9
        let mut n = first.to_digit(10).unwrap();
        let mut count = 1;

        while let Some(peek) = self.src.peek().cloned() {
            if !peek.is_digit(10) {
                break;
            }

            // If next() was None, the `while let ...` would never get here
            let next = self.next_char().unwrap();

            // TODO: What happens when `n` overflows
            n = n * 10 + next.to_digit(10).unwrap();
            count += 1;
        }

        (n, count)
    }

    fn tokenize_alpha(&mut self, first: char, span: Span) -> Result<Token> {
        let kind = match first {
            'G' => TokenKind::G,
            'M' => TokenKind::M,
            'T' => TokenKind::T,
            'N' => TokenKind::N,

            'X' => TokenKind::X,
            'Y' => TokenKind::Y,
            'Z' => TokenKind::Z,
            'R' => TokenKind::R,
            'F' => TokenKind::FeedRate,
            'O' => TokenKind::O,

            other => {
                debug!("Using escape hatch for character: {}", other);
                TokenKind::Other(other)
            }
        };

        Ok(Token { kind, span })
    }

    fn to_end_of_line(&mut self) {
        while let Some(peek) = self.src.peek().cloned() {
            if peek == '\n' {
                let _ = self.next_char();
                break;
            }

            let _ = self.next_char();
        }
    }

    fn skip_comment(&mut self) {
        while self.src.peek().map_or(false, |&peek| peek != ')') {
            let _ = self.next_char();
        }

        let _ = self.next_char();
    }
}


impl<I> Iterator for Tokenizer<I>
    where I: Iterator<Item = char>
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.next_token();
        if let Some(tok) = t.as_ref() {
            trace!("{:?}", tok);
        }

        t
    }
}


/// A gcode Token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    /// Which kind of token is this?
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Get the location of the token in the source code.
    pub fn span(&self) -> Span {
        self.span
    }
}


/// A `gcode` token.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum TokenKind {
    /// A floating point number.
    Number(f32),
    /// A plain integer.
    Integer(u32),

    G,
    M,
    T,
    N,
    /// A program number.
    O,

    X,
    Y,
    Z,
    FeedRate,
    R,

    Minus,
    Percent,

    /// An escape hatch which matches any other single alphabetic character.
    ///
    /// # Note
    ///
    /// This probably shouldn't be used outside the crate, if you end up
    /// matching on a TokenKind::Other chances are you need to amend the
    /// `TokenKind` definition.
    #[doc(hidden)]
    Other(char),
}


/// A representation of a position in source code.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Span {
    /// The line number (counting from zero).
    pub line: usize,
    /// The column number (counting from zero).
    pub column: usize,
}


impl From<(usize, usize)> for Span {
    fn from(other: (usize, usize)) -> Self {
        Span {
            line: other.0,
            column: other.1,
        }
    }
}

impl PartialEq<TokenKind> for Token {
    fn eq(&self, other: &TokenKind) -> bool {
        self.kind == *other
    }
}

impl From<TokenKind> for Token {
    fn from(other: TokenKind) -> Self {
        Token {
            kind: other,
            span: Span::default(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_single_letter_tokens() {
        let inputs = [("G", TokenKind::G),
                      ("M", TokenKind::M),
                      ("T", TokenKind::T),
                      ("N", TokenKind::N),

                      ("X", TokenKind::X),
                      ("Y", TokenKind::Y),
                      ("Z", TokenKind::Z),
                      ("R", TokenKind::R),
                      ("F", TokenKind::FeedRate),

                      ("w", TokenKind::Other('w'))];

        for &(src, should_be) in &inputs {
            let mut tokenizer = Tokenizer::new(src.chars());
            let first = tokenizer.next_token().unwrap().unwrap();

            assert_eq!(first, should_be);
        }
    }

    #[test]
    fn tokenize_numbers() {
        let inputs = [("100000000", TokenKind::Integer(100000000)),
                      ("0", TokenKind::Integer(0)),
                      ("12", TokenKind::Integer(12)),
                      ("12.", TokenKind::Number(12.0)),
                      ("12.34", TokenKind::Number(12.34)),
                      ("00012312.00000001", TokenKind::Number(12312.00000001)),
                      ("12.34.", TokenKind::Number(12.34))];

        for &(src, should_be) in &inputs {
            println!("{} => {:?}", src, should_be);
            let mut tokenizer = Tokenizer::new(src.chars());
            let first = tokenizer.next_token().unwrap().unwrap();

            assert_eq!(first, should_be);
        }
    }

    #[test]
    fn tokenize_integers() {
        let inputs = [("12", (12, 2)),
                      ("1", (1, 1)),
                      ("12.34", (12, 2)),
                      ("12.34.", (12, 2))];

        for &(src, should_be) in &inputs {
            let mut tokenizer = Tokenizer::new(src.chars());
            let next = tokenizer.src.next().unwrap();
            let first = tokenizer.tokenize_integer(next);

            assert_eq!(first, should_be);
        }
    }

    #[test]
    fn tokenizer_skips_comments() {
        let src = "(hello world)7";
        let mut tokenizer = Tokenizer::new(src.chars());
        tokenizer.skip_comment();
        assert_eq!(tokenizer.src.next(), Some('7'));
    }

    #[test]
    fn tokenizer_skips_to_end_of_line() {
        let src = "awleifr 238r\n7";
        let mut tokenizer = Tokenizer::new(src.chars());
        tokenizer.to_end_of_line();
        assert_eq!(tokenizer.src.next(), Some('7'));
    }

    #[test]
    fn tokenize_program_number() {
        let src = "O";
        let should_be = TokenKind::O;

        let got = Tokenizer::new(src.chars()).next().unwrap().unwrap();
        assert_eq!(got, should_be);
    }
}
