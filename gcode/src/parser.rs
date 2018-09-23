use core::iter::Peekable;
use lexer::{Lexer, Token};
#[cfg(not(feature = "std"))]
use libm::F32Ext;
use types::{Argument, Block, Comment, Gcode, Mnemonic, Span};

#[derive(Debug, Clone)]
pub struct Parser<'input, C> {
    src: &'input str,
    lexer: Peekable<Lexer<'input>>,
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
            lexer: Lexer::new(src).peekable(),
            src,
            callbacks,
        }
    }

    /// Access the inner `Callbacks` object.
    pub fn callbacks(&mut self) -> &mut C {
        &mut self.callbacks
    }

    fn parse_block(&mut self) -> Option<Block<'input>> {
        let mut block = Block::empty();

        self.parse_preamble(&mut block);
        self.parse_commands(&mut block);

        if block.is_empty() {
            None
        } else {
            let src = block
                .span()
                .text_from_source(self.src)
                .expect("The span should always be valid");
            block.with_src(src);

            Some(block)
        }
    }

    fn parse_preamble(&mut self, block: &mut Block<'input>) {
        while let Some(&(token, span)) = self.lexer.peek() {
            match token {
                Token::ForwardSlash => {
                    let _ = self.lexer.next();
                    block.delete(true);
                }
                Token::Comment(body) => {
                    let _ = self.lexer.next();
                    block.push_comment(Comment { body, span });
                }
                Token::Letter(n) if n == 'n' || n == 'N' => {
                    let _ = self.lexer.next();

                    if let Some(arg) = self.parse_word(n, span, block) {
                        block.with_line_number(arg.value as usize, span);
                    }
                }
                Token::Letter(_) => return,
                _ => unimplemented!(),
            }
        }
    }

    fn parse_word(
        &mut self,
        letter: char,
        letter_span: Span,
        block: &mut Block<'input>,
    ) -> Option<Argument> {
        let (tok, span) = self.chomp(Token::NUMBER, block)?;

        let value = match tok {
            Token::Number(n) => n,
            other => unreachable!(
                "We've already checked and {:?} should be a number",
                other
            ),
        };

        Some(Argument {
            letter,
            value,
            span: span.merge(letter_span),
        })
    }

    fn parse_commands(&mut self, block: &mut Block<'input>) {
        while let Some(&(token, span)) = self.lexer.peek() {
            match token {
                Token::Newline => return,
                Token::Comment(body) => {
                    let _ = self.lexer.next();
                    block.push_comment(Comment { body, span });
                }
                Token::Letter(letter) => {
                    self.parse_command(block);
                }
                other => {
                    self.callbacks.unexpected_token(
                        other.kind(),
                        span,
                        &[Token::LETTER, Token::COMMENT, Token::NEWLINE],
                    );
                    let _ = self.lexer.next();
                }
            }
        }
    }

    fn parse_command(&mut self, block: &mut Block<'input>) {
        let (tok, mut span) = self.lexer.next().expect("Already checked");

        let letter = match tok {
            Token::Letter(l) => l,
            other => unreachable!("{:?} should only ever be a letter"),
        };

        let (number, number_span) = match self.chomp(Token::NUMBER, block) {
            Some((Token::Number(n), span)) => (n, span),
            Some(other) => {
                unreachable!("Chomp ensures {:?} is a number", other)
            }
            None => return,
        };
        span = span.merge(number_span);

        let mnemonic = match letter {
            'g' | 'G' => Mnemonic::General,
            'm' | 'M' => Mnemonic::Miscellaneous,
            other => unimplemented!(
                "Found {}. What happens when command names are elided?",
                other,
            ),
        };

        let mut cmd = Gcode::new(mnemonic, number);
        cmd.with_span(span);
        block.push_command(cmd);
    }

    /// Look ahead at the next token, advancing and returning the token if it
    /// is the correct type (`kind`). Any comments will automatically be added
    /// to the block.
    fn chomp(
        &mut self,
        kind: &'static str,
        block: &mut Block<'input>,
    ) -> Option<(Token<'input>, Span)> {
        while let Some(&(token, span)) = self.lexer.peek() {
            if let Token::Comment(body) = token {
                block.push_comment(Comment { body, span });
                let _ = self.lexer.next();
                continue;
            }

            if token.kind() != kind {
                self.callbacks.unexpected_token(token.kind(), span, &[kind]);
                return None;
            } else {
                return self.lexer.next();
            }
        }

        None
    }
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
        _found: &str,
        _span: Span,
        _expected: &[&str],
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
        assert_eq!(g90.major_number(), 90);
        assert!(g90.args().is_empty());
    }

    #[test]
    fn read_a_deleted_g90() {
        let mut parser = Parser::new("/N20 G90");

        let block = parser.next().unwrap();

        assert_eq!(block.line_number(), Some(20));
        assert!(block.comments().is_empty());
        assert!(block.deleted());

        assert_eq!(block.commands().len(), 1);
    }
}
