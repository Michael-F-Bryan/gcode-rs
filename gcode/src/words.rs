use crate::lexer::{Lexer, Token, TokenType};
use crate::{Comment, Span};

/// A [`char`]-[`f32`] pair, used for things like arguments (`X3.14`), command
/// numbers (`G90`) and line numbers (`N10`).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Word {
    /// The letter part of this [`Word`].
    pub letter: char,
    /// The value part.
    pub value: f32,
    /// Where the [`Word`] lies in the original string.
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Atom<'input> {
    Word(Word),
    Comment(Comment<'input>),
    /// Incomplete parts of a [`Word`].
    BrokenWord(Token<'input>),
    /// Garbage from the tokenizer (see [`TokenType::Unknown`]).
    Unknown(Token<'input>),
}

impl<'input> Atom<'input> {
    pub(crate) fn span(&self) -> Span {
        match self {
            Atom::Word(word) => word.span,
            Atom::Comment(comment) => comment.span,
            Atom::Unknown(token) | Atom::BrokenWord(token) => token.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WordsOrComments<I> {
    tokens: I,
}

impl<'input, I> WordsOrComments<I>
where
    I: Iterator<Item = Token<'input>>,
{
    pub fn new(tokens: I) -> Self {
        WordsOrComments { tokens }
    }
}

impl<'input, I> Iterator for WordsOrComments<I>
where
    I: Iterator<Item = Token<'input>>,
{
    type Item = Atom<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        // keep track of the last letter so we can deal with a trailing letter
        // that has no number
        let mut last_letter: Option<Token> = None;

        while let Some(token) = self.tokens.next() {
            let Token { kind, value, span } = token;

            match kind {
                TokenType::Unknown => return Some(Atom::Unknown(token)),
                TokenType::Comment => return Some(Atom::Comment(Comment { value, span })),
                TokenType::Letter if last_letter.is_none() => {
                    last_letter = Some(token);
                }
                TokenType::Number if last_letter.is_some() => {
                    let letter_token = last_letter.unwrap();
                    let span = letter_token.span.merge(span);

                    debug_assert_eq!(letter_token.value.len(), 1);
                    let letter = letter_token.value.chars().next().unwrap();
                    let value = value.parse().expect("");

                    return Some(Atom::Word(Word {
                        letter,
                        value,
                        span,
                    }));
                }
                _ => return Some(Atom::BrokenWord(token)),
            }
        }

        last_letter.map(Atom::BrokenWord)
    }
}

impl<'input> From<&'input str> for WordsOrComments<Lexer<'input>> {
    fn from(other: &'input str) -> WordsOrComments<Lexer<'input>> {
        WordsOrComments::new(Lexer::new(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn pass_comments_through() {
        let mut words = WordsOrComments::new(Lexer::new("(this is a comment) 3.14"));

        let got = words.next().unwrap();

        let comment = "(this is a comment)";
        let expected = Atom::Comment(Comment {
            value: comment,
            span: Span {
                start: 0,
                end: comment.len(),
                line: 0,
            },
        });
        assert_eq!(got, expected);
    }

    #[test]
    fn pass_garbage_through() {
        let text = "!@#$ *";
        let mut words = WordsOrComments::new(Lexer::new(text));

        let got = words.next().unwrap();

        let expected = Atom::Unknown(Token {
            value: text,
            kind: TokenType::Unknown,
            span: Span {
                start: 0,
                end: text.len(),
                line: 0,
            },
        });
        assert_eq!(got, expected);
    }

    #[test]
    fn numbers_are_garbage_if_they_dont_have_a_letter_in_front() {
        let text = "3.14 ()";
        let mut words = WordsOrComments::new(Lexer::new(text));

        let got = words.next().unwrap();

        let expected = Atom::BrokenWord(Token {
            value: "3.14",
            kind: TokenType::Number,
            span: Span {
                start: 0,
                end: 4,
                line: 0,
            },
        });
        assert_eq!(got, expected);
    }

    #[test]
    fn recognise_a_valid_word() {
        let text = "G90";
        let mut words = WordsOrComments::new(Lexer::new(text));

        let got = words.next().unwrap();

        let expected = Atom::Word(Word {
            letter: 'G',
            value: 90.0,
            span: Span {
                start: 0,
                end: text.len(),
                line: 0,
            },
        });
        assert_eq!(got, expected);
    }
}
