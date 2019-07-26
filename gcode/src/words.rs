use crate::lexer::{Token, TokenType};
use crate::{Comment, Span};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Word {
    pub letter: char,
    pub value: f32,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Atom<'input> {
    Word(Word),
    Comment(Comment<'input>),
    UnexpectedLetter(Token<'input>),
    Unknown(Token<'input>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WordsOrComments<'input, I: 'input> {
    tokens: I,
    last_letter: Option<Token<'input>>,
}

impl<'input, I> WordsOrComments<'input, I>
where
    I: Iterator<Item = Token<'input>>,
{
    pub fn new(tokens: I) -> Self {
        WordsOrComments {
            tokens,
            last_letter: None,
        }
    }
}

impl<'input, I> Iterator for WordsOrComments<'input, I>
where
    I: Iterator<Item = Token<'input>>,
{
    type Item = Atom<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        // keep track of the last token so we can deal with a trailing letter
        // that has no number
        let last_token = None;

        while let Some(token) = self.tokens.next() {
            match token.kind {
                TokenType::Unknown => unimplemented!(),
                _ => unimplemented!(),
            }
        }

        last_token.map(Atom::UnexpectedLetter)
    }
}
