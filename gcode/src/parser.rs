use lexer::{Lexer, Token};
#[cfg(not(feature = "std"))]
use libm::F32Ext;
use types::{Block, Comment, Mnemonic, Span};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Parser<'input, C> {
    lexer: Lexer<'input>,
    callbacks: C,
    state: State,
}

impl<'input> Parser<'input, Nop> {
    pub fn new(src: &'input str) -> Parser<'input, Nop> {
        Parser::new_with_callbacks(src, Nop)
    }
}

impl<'input, C: Callbacks> Parser<'input, C> {
    pub fn new_with_callbacks(
        src: &'input str,
        callbacks: C,
    ) -> Parser<'input, C> {
        Parser {
            lexer: Lexer::new(src),
            callbacks,
            state: State::Preamble,
        }
    }

    fn parse_block(&mut self) -> Option<Block<'input>> {
        let mut block = Block::empty();
        self.state = State::Preamble;

        while let Some((token, span)) = self.lexer.next() {
            match self.state {
                State::Preamble => {
                    self.step_start(token, span, &mut block);
                }
                State::ReadingLineNumber(n_span) => {
                    self.step_read_line_number(token, span, n_span, &mut block);
                }
                State::ReadingWord(letter, letter_span) => {
                    self.step_read_word(
                        token,
                        span,
                        letter,
                        letter_span,
                        &mut block,
                    );
                }
                State::Done => {}
            }

            if self.state == State::Done {
                break;
            }
        }

        if block.is_empty() {
            None
        } else {
            if let Some(s) = block.span().text_from_source(self.lexer.src()) {
                block.with_src(s);
            }

            Some(block)
        }
    }

    fn step_start(
        &mut self,
        token: Token<'input>,
        span: Span,
        block: &mut Block<'input>,
    ) {
        match token {
            Token::Comment(body) => {
                block.push_comment(Comment { body, span });
            }
            Token::Newline => {
                if block.is_empty() {
                    // ignore it
                } else {
                    self.state = State::Done;
                }
            }
            Token::ForwardSlash => {
                if block.is_empty() {
                    block.delete(true);
                } else {
                    self.callbacks.unexpected_token(
                        token.kind(),
                        span,
                        &[Token::COMMENT, Token::NEWLINE, Token::LETTER],
                    );
                }
            }
            Token::Letter('N') | Token::Letter('n') => {
                self.state = State::ReadingLineNumber(span);
            }
            Token::Letter(other) => {
                self.state = State::ReadingWord(other, span);
            }
            _ => unimplemented!(),
        }
    }

    fn step_read_word(
        &mut self,
        token: Token<'input>,
        token_span: Span,
        letter: char,
        letter_span: Span,
        block: &mut Block<'input>,
    ) {
        let number = match token {
            Token::Number(n) => n,
            other => {
                self.callbacks.unexpected_token(
                    other.kind(),
                    token_span,
                    &[Token::NUMBER],
                );
                unimplemented!();
            }
        };

        unimplemented!();
    }

    fn step_read_line_number(
        &mut self,
        token: Token<'input>,
        token_span: Span,
        n_span: Span,
        block: &mut Block<'input>,
    ) {
        match token {
            Token::Number(line_number) => {
                block.with_line_number(
                    line_number.abs().trunc() as usize,
                    token_span.merge(n_span),
                );
            }
            _ => {
                self.callbacks.unexpected_token(
                    token.kind(),
                    token_span,
                    &[Token::NUMBER],
                );
            }
        }

        self.state = State::Preamble;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    /// We're reading the stuff at the beginning of a line.
    Preamble,
    /// Started reading a line number.
    ReadingLineNumber(Span),
    /// We're reading a word (i.e. `G90`).
    ReadingWord(char, Span),
    /// Finished reading a line.
    Done,
}

impl<'input, C: Callbacks> Iterator for Parser<'input, C> {
    type Item = Block<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_block()
    }
}

pub trait Callbacks {
    fn unexpected_token(
        &mut self,
        _found: &'static str,
        _span: Span,
        _expected: &'static [&'static str],
    ) {
    }
    fn unexpected_eof(&mut self, _expected: &[&str]) {}
}

/// A no-op set of callbacks.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nop;

impl Callbacks for Nop {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_a_comment_block() {
        let src = "; This is a comment\n";
        let mut parser = Parser::new(src);

        let block = parser.next().unwrap();

        assert!(block.commands().is_empty());
        assert_eq!(block.comments().len(), 1);

        let comment = &block.comments()[0];
        assert_eq!(comment.body, "; This is a comment");
        assert_eq!(
            comment.span,
            Span {
                start: 0,
                end: src.len(),
                source_line: 0
            }
        );
        assert_eq!(block.span(), comment.span);
    }

    #[test]
    fn read_a_line_number() {
        let mut parser = Parser::new("N42");

        let block = parser.next().unwrap();

        assert_eq!(block.line_number(), Some(42));
        assert!(block.comments().is_empty());
        assert!(block.commands().is_empty());
    }

    #[test]
    fn read_a_g90() {
        let mut parser = Parser::new("G90");

        let block = parser.next().unwrap();

        assert!(block.line_number().is_none());
        assert!(block.comments().is_empty());

        assert_eq!(block.commands().len(), 1);
        let g90 = &block.commands()[0];

        assert_eq!(g90.mnemonic(), Mnemonic::General);
        assert!(g90.args().is_empty());
    }
}
