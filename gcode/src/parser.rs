use arrayvec::ArrayVec;
use lexer::{Lexer, Token};
use types::{Block, Comment, Span};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Parser<'input, C> {
    lexer: Lexer<'input>,
    callbacks: C,
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
        }
    }

    fn parse_block(&mut self) -> Option<Block<'input>> {
        let mut block = Block::empty();

        while let Some((token, span)) = self.lexer.next() {
            match token {
                Token::Newline => {
                    if block.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }
                Token::ForwardSlash => {
                    if block.is_empty() {
                        block.delete(true);
                    } else {
                        self.callbacks.unexpected_block_delete(span);
                    }
                }
                Token::Comment(body) => {
                    block.push_comment(Comment { body, span });
                }
                _ => unimplemented!(),
            }
        }

        if block.is_empty() {
            None
        } else {
            if let Some(s) = block.span().text_from_source(self.lexer.src()) {
                block.set_src(s);
            }

            Some(block)
        }
    }
}

impl<'input, C: Callbacks> Iterator for Parser<'input, C> {
    type Item = Block<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_block()
    }
}

pub trait Callbacks {
    fn unexpected_block_delete(&mut self, span: Span) {}
}

/// A no-op set of callbacks.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nop;

impl Callbacks for Nop {}

#[derive(Debug, Clone, PartialEq)]
struct Lookahead<I: Iterator> {
    iter: I,
    buffer: ArrayVec<[I::Item; 3]>,
}

impl<I: Iterator> Iterator for Lookahead<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.buffer.is_empty() {
            Some(self.buffer.remove(0))
        } else {
            self.iter.next()
        }
    }
}

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
}
