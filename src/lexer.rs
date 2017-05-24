use core::iter::Peekable;

use errors::*;
use helpers::*;


/// A zero-allocation tokenizer.
///
/// # Examples
///
/// ```rust
/// use gcode::{Token, Tokenizer};
/// let should_be = [Token::N,
///                  Token::Integer(40),
///                  Token::G,
///                  Token::Integer(90),
///                  Token::X,
///                  Token::Number(1.0)];
///
/// let src = "N40 G90 X1.0";
/// let tokens = Tokenizer::new(src.chars());
///
/// assert!(tokens.zip(&should_be)
///               .all(|(got, &should_be)| got.unwrap() == should_be));
/// ```
pub struct Tokenizer<I>
    where I: Iterator<Item = char>
{
    src: Peekable<I>,
}


impl<I> Tokenizer<I>
    where I: Iterator<Item = char>
{
    /// Create a new `Tokenizer` from some `char` iterator.
    pub fn new(src: I) -> Self {
        Tokenizer { src: src.peekable() }
    }


    fn next_token(&mut self) -> Option<Result<Token>> {
        while let Some(peek) = self.src.next() {
            if peek.is_whitespace() {
                continue;
            }

            let tok = match peek {
                d if d.is_digit(10) || d == '.' => self.tokenize_number(d),
                a if a.is_alphabetic() => self.tokenize_alpha(a),

                ';' => {
                    self.to_end_of_line();
                    continue;
                }
                '(' => {
                    self.skip_comment();
                    continue;
                }

                other => Err(Error::UnknownToken(other)),
            };

            return Some(tok);
        }

        None
    }

    fn tokenize_number(&mut self, first: char) -> Result<Token> {
        // TODO: Make clean... pls
        let (integer_part, _) = self.tokenize_integer(first);

        match self.src.peek() {
            Some(&'.') => {}
            _ => return Ok(Token::Integer(integer_part)),
        }

        let _ = self.src.next();

        let t = match self.src.peek().cloned() {
            Some(d) if d.is_digit(10) => {
                let next = self.src.next().unwrap();
                let (fractional_part, length) = self.tokenize_integer(next);

                let number = float_from_integers(integer_part, fractional_part, length);
                Token::Number(number)
            }
            _ => Token::Number(integer_part as f32),
        };

        Ok(t)
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
            let next = self.src.next().unwrap();

            // TODO: What happens when `n` overflows
            n = n * 10 + next.to_digit(10).unwrap();
            count += 1;
        }

        (n, count)
    }

    fn tokenize_alpha(&mut self, first: char) -> Result<Token> {
        let t = match first {
            'G' => Token::G,
            'M' => Token::M,
            'T' => Token::T,
            'N' => Token::N,

            'X' => Token::X,
            'Y' => Token::Y,
            'Z' => Token::Z,
            'R' => Token::R,
            'F' => Token::FeedRate,

            other => Token::Other(other),
        };

        Ok(t)
    }

    fn to_end_of_line(&mut self) {
        while let Some(peek) = self.src.peek().cloned() {
            if peek == '\n' {
                let _ = self.src.next();
                break;
            }

            let _ = self.src.next();
        }
    }

    fn skip_comment(&mut self) {
        let open_paren = self.src.next();
        debug_assert!(open_paren == Some('('),
                      "A comment should always start with a '('");

        while self.src.peek().map_or(false, |&peek| peek != ')') {
            let _ = self.src.next();
        }

        let _ = self.src.next();
    }
}


/// A `gcode` token.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Token {
    Number(f32),
    Integer(u32),

    G,
    M,
    T,
    N,

    X,
    Y,
    Z,
    FeedRate,
    R,

    Other(char),
}

impl<I> Iterator for Tokenizer<I>
    where I: Iterator<Item = char>
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_single_letter_tokens() {
        let inputs = [("G", Token::G),
                      ("M", Token::M),
                      ("T", Token::T),
                      ("N", Token::N),

                      ("X", Token::X),
                      ("Y", Token::Y),
                      ("Z", Token::Z),
                      ("R", Token::R),
                      ("F", Token::FeedRate),

                      ("w", Token::Other('w'))];

        for &(src, should_be) in &inputs {
            let mut tokenizer = Tokenizer::new(src.chars());
            let first = tokenizer.next_token().unwrap().unwrap();

            assert_eq!(first, should_be);
        }
    }

    #[test]
    fn tokenize_numbers() {
        let inputs = [("100000000", Token::Integer(100000000)),
                      ("0", Token::Integer(0)),
                      ("12", Token::Integer(12)),
                      ("12.", Token::Number(12.0)),
                      ("12.34", Token::Number(12.34)),
                      ("00012312.00000001", Token::Number(12312.00000001)),
                      ("12.34.", Token::Number(12.34))];

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
}
