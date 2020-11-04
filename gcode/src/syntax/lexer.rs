use crate::Span;

/// Split some text into its constituent tokens.
pub fn tokenize(text: &str) -> impl Iterator<Item = Token<'_>> + '_ {
    Tokenizer::new(text)
}

pub(crate) struct Tokenizer<'a> {
    src: &'a str,
    current_index: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a str) -> Self {
        Tokenizer {
            src,
            current_index: 0,
        }
    }

    fn rest(&self) -> &'a str { &self.src[self.current_index..] }

    fn is_done(&self) -> bool { self.rest().is_empty() }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            return None;
        }

        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token<'a> {
    raw: &'a str,
    span: Span,
    value: TokenValue<'a>,
}

impl<'a> Token<'a> {
    pub(crate) fn new(raw: &'a str, span: Span, value: TokenValue<'a>) -> Self {
        Token { raw, span, value }
    }

    pub const fn raw(&self) -> &'a str { self.raw }

    pub const fn span(&self) -> Span { self.span }

    pub const fn value(&self) -> TokenValue<'a> { self.value }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenValue<'a> {
    /// An ascii character.
    Character(char),
    /// An integer.
    Integer(u32),
    /// A decimal number.
    Float(f64),
    /// A comment.
    Comment(&'a str),
    /// A percent character.
    Percent,
    /// Skip everything from here to the end of the line.
    BlockSkip,
    /// Characters which can't be recognised by the tokenizer.
    Unknown,
    /// A newline character.
    Newline,
}
