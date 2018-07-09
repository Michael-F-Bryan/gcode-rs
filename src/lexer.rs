use number::{Number, Prescalar};
use types::{Span, Word};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Lexer<'input> {
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

    fn skip(&mut self) -> usize {
        let mut total_skipped = 0;

        loop {
            let bytes_skipped = self.skip_whitespace() + self.skip_comments();
            if bytes_skipped == 0 {
                break;
            } else {
                total_skipped += bytes_skipped;
            }
        }

        total_skipped
    }

    fn skip_whitespace(&mut self) -> usize {
        self.take_while(|c| c.is_whitespace()).len()
    }

    fn skip_comments(&mut self) -> usize {
        match self.peek() {
            Some('(') => {
                // consume up to the closing paren
                let skipped = self.take_while(|c| c != ')');
                // then consume the paren itself
                self.advance();
                skipped.len() + 1
            }
            Some(';') => {
                // skip until the end of the line
                self.take_while(|c| c != '\n' && c != '\r').len()
            }
            _ => 0,
        }
    }

    fn read_integer(&mut self) -> Option<u32> {
        let read = self.take_while(|c| c.is_digit(10));

        if read.is_empty() {
            None
        } else {
            // FIXME: what happens if the number is too long?
            Some(read.parse().expect("Should never fail"))
        }
    }

    fn read_mnemonic(&mut self) -> Option<char> {
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.advance();
                Some(c.to_ascii_uppercase())
            }
            _ => None,
        }
    }

    fn read_number<P: Prescalar + Default>(&mut self) -> Option<Number<P>> {
        self.try_or_backtrack(|lexy| {
            let start = lexy.current_index;
            let integral_part = lexy.read_integer()?;

            if lexy.peek() == Some('.') {
                lexy.advance();
                lexy.read_integer();
            }

            let number = lexy.src[start..lexy.current_index]
                .parse()
                .expect("Parse always succeeds");
            Some(number)
        })
    }

    fn read_word(&mut self) -> Option<Word> {
        self.try_or_backtrack(|lexy| {
            let start = lexy.current_index;
            let start_line = lexy.current_line;
            let letter = lexy.read_mnemonic()?;
            let number = lexy.read_number()?;

            let span = Span::new(start, lexy.current_index, start_line);
            Some(Word {
                letter,
                number,
                span,
            })
        })
    }

    /// Tries to tokenize a thing. If the tokenizing fails then reset the
    /// `current_index` back to its initial value.
    fn try_or_backtrack<F, T>(&mut self, thunk: F) -> Option<T>
    where
        F: FnOnce(&mut Lexer) -> Option<T>,
    {
        let start = self.current_index;

        let got = thunk(self);
        if got.is_none() {
            self.current_index = start;
        }

        got
    }

    fn take_while<F>(&mut self, mut predicate: F) -> &'input str
    where
        F: FnMut(char) -> bool,
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

    fn remaining(&self) -> &str {
        &self.src[self.current_index..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.current_index += c.len_utf8();
            if c == '\n' {
                self.current_line += 1;
            }
        }
    }

    fn finished(&self) -> bool {
        self.remaining().is_empty()
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.finished() {
            self.skip();

            match self.read_word() {
                Some(word) => return Some(word),
                // couldn't find anything. Let's step past it and try again
                None => self.advance(),
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_whitespace() {
        let mut lexy = Lexer::new("   ");
        assert!(!lexy.finished());

        lexy.skip_whitespace();

        assert!(lexy.finished());
    }

    #[test]
    fn tokenize_a_number() {
        let mut lexy = Lexer::new("123");

        let got = lexy.read_integer().unwrap();

        assert_eq!(got, 123);
    }

    #[test]
    fn tokenize_a_character() {
        let inputs = vec![
            ("G123", Some('G')),
            ("A", Some('A')),
            ("a", Some('A')),
            ("$", None),
            ("123", None),
            (" ", None),
        ];

        for (src, should_be) in inputs {
            let mut lexy = Lexer::new(src);
            let got = lexy.read_mnemonic();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn skip_a_bracket_comment() {
        let src = "(this is a comment)";
        let mut lexy = Lexer::new(src);

        lexy.skip_comments();

        assert!(lexy.finished());
    }

    #[test]
    fn skip_a_line_comment() {
        let src = ";this is a comment\n";
        let mut lexy = Lexer::new(src);

        lexy.skip_comments();

        assert_eq!(lexy.current_index, src.len() - 1);
    }

    #[test]
    fn skip_a_line_comment_without_trailing_newline() {
        let src = ";this is a comment";
        let mut lexy = Lexer::new(src);

        lexy.skip_comments();

        assert!(lexy.finished());
    }

    #[test]
    fn tokenize_a_word() {
        let src = "G01";
        let should_be = Word {
            letter: 'G',
            number: Number::from(1),
            span: Span::new(0, src.len(), 0),
        };

        let got = Lexer::new(src).next().unwrap();

        assert_eq!(got, should_be);
    }
}
