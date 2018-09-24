use core::iter::Peekable;
use lexer::{Lexer, Token, TokenKind};
#[cfg(not(feature = "std"))]
use libm::F32Ext;
use types::{Argument, Block, Comment, Gcode, Mnemonic, Span};

#[derive(Debug, Clone)]
/// An error-resistent streaming gcode parser.
///
/// # Grammar
///
/// The language grammar and parser for gcode isn't especially complicated. Different manufacturers
/// use slightly different dialects though, so the grammar needs to be flexible
/// and accept input which might otherwise be interpreted as erroneous.
///
/// Any possible errors are signalled to the caller via the [`Callbacks`]
/// object.
///
/// The gcode language looks something like the following pseudo-ebnf:
///
/// ```ebnf
/// program       := block*
/// block         := block-delete? line-number? (command | comment)+ NEWLINE
///                | percent-line
/// percent-line  := "%" comment?
/// line-number   := "n" NUMBER
/// block-delete  := "/"
/// comment       := "(" TEXT ")"
///                | ";" TEXT "\n"
/// command       := mnemonic NUMBER word*
///                | word
/// word          := argument NUMBER
///
/// mnemonic      := "g" | "m" | "o" | "t"
/// argument      := /* all letters except those in mnemonic */
/// ```
///
/// > **Note:**
/// > - A comment may occur anywhere in the input and should be attached to the
/// >   nearest block
/// > - All letters are case-insensitive
///
/// See also [Constructing human-grade parsers].
///
/// [`Callbacks`]: trait.Callbacks.html
/// [Constructing human-grade parsers]: http://duriansoftware.com/joe/Constructing-human-grade-parsers.html
pub struct Parser<'input, C> {
    src: &'input str,
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
        while let Some(kind) = self.next_kind() {
            match kind {
                TokenKind::Comment => {
                    unreachable!("next_kind() never yields a comment")
                }
                TokenKind::ForwardSlash => {
                    let _ = self
                        .chomp(kind, |c| block.push_comment(c))
                        .expect("Already checked");
                    block.delete(true);
                }
                TokenKind::Newline => {
                    if block.is_empty() {
                        let _ = self.chomp(TokenKind::Newline, |c| {
                            block.push_comment(c)
                        });
                        continue;
                    } else {
                        return;
                    }
                }
                TokenKind::Letter => {
                    if self.next_is_letter_n() {
                        self.parse_line_number(block);
                        continue;
                    } else {
                        // It's something else. Break out of the loop so we
                        // can start parsing commands
                        return;
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    /// Try to read a line number (`N42`) and set the block's line number if
    /// successful.
    fn parse_line_number(&mut self, block: &mut Block<'input>) {
        let (tok, span) = self
            .chomp(TokenKind::Letter, |c| block.push_comment(c))
            .expect("Already checked");

        let l = tok.unwrap_letter();
        match self.parse_word(l, span, |c| block.push_comment(c)) {
            Some(word) => {
                block.with_line_number(word.value as usize, word.span);
            }
            // we found a "N", but it wasn't followed by a number
            None => unimplemented!(),
        }
    }

    fn next_is_letter_n(&self) -> bool {
        self.lookahead(|lexy| {
            for (tok, _) in lexy {
                match tok {
                    Token::Letter('n') | Token::Letter('N') => return true,
                    Token::Comment(_) => continue,
                    _ => return false,
                }
            }

            false
        })
    }

    fn parse_word(
        &mut self,
        letter: char,
        letter_span: Span,
        comments: impl FnMut(Comment<'input>),
    ) -> Option<Argument> {
        let (tok, span) = self.chomp(TokenKind::Number, comments)?;
        let value = tok.unwrap_number();

        Some(Argument {
            letter,
            value,
            span: span.merge(letter_span),
        })
    }

    fn parse_commands(&mut self, block: &mut Block<'input>) {
        while let Some(next) = self.next_kind() {
            if next == TokenKind::Newline {
                return;
            }

            match self.parse_command(|c| block.push_comment(c)) {
                Some(cmd) => block.push_command(cmd),
                None => {
                    self.fast_forward_to_safe_point(|c| block.push_comment(c))
                }
            }
        }
    }

    /// Something went wrong while trying to read a gcode command. Fast forward
    /// until we see the start of a new command or the end of the block.
    fn fast_forward_to_safe_point(
        &mut self,
        comments: impl FnMut(Comment<'input>),
    ) {
        unimplemented!()
    }

    fn parse_command(
        &mut self,
        mut comments: impl FnMut(Comment<'input>),
    ) -> Option<Gcode> {
        let (tok, mut span) = self.chomp(TokenKind::Letter, &mut comments)?;

        let letter = match tok {
            Token::Letter(l) => l,
            other => unreachable!("{:?} should only ever be a letter", other),
        };

        let (number, number_span) = self.chomp(TokenKind::Number, comments)?;
        let number = match number {
            Token::Number(n) => n,
            _ => unreachable!(),
        };

        span = span.merge(number_span);

        let mnemonic = match letter {
            'g' | 'G' => Mnemonic::General,
            'm' | 'M' => Mnemonic::Miscellaneous,
            'o' | 'O' => Mnemonic::ProgramNumber,
            't' | 'T' => Mnemonic::ToolChange,
            other => unimplemented!(
                "Found {}. What happens when command names are elided?",
                other,
            ),
        };

        let mut cmd = Gcode::new(mnemonic, number);
        cmd.with_span(span);

        // TODO: read the arguments and notify the callbacks if there was an error

        Some(cmd)
    }

    /// Scan forward and see the `TokenKind` for the next non-comment `Token`.
    fn next_kind(&self) -> Option<TokenKind> {
        self.lookahead(|lexy| {
            lexy.map(|(tok, _)| tok.kind())
                .filter(|&kind| kind != TokenKind::Comment)
                .next()
        })
    }

    fn lookahead<F, T>(&self, peek: F) -> T
    where
        F: FnOnce(Lexer) -> T,
    {
        peek(self.lexer.clone())
    }

    /// Look ahead at the next token, advancing and returning the token if it
    /// is the correct type (`kind`). Any comments will automatically be added
    /// to the block.
    fn chomp(
        &mut self,
        kind: TokenKind,
        mut comments: impl FnMut(Comment<'input>),
    ) -> Option<(Token<'input>, Span)> {
        // Look ahead and make sure the next non-comment token is the one we
        // want
        if self.next_kind() != Some(kind) {
            return None;
        }

        while let Some((tok, span)) = self.lexer.next() {
            // We found it!
            if tok.kind() == kind {
                return Some((tok, span));
            }

            if let Token::Comment(body) = tok {
                comments(Comment { body, span });
            } else {
                unreachable!(
                    "We should only ever see a {} or comments. Found {:?}",
                    kind, tok
                );
            }
        }

        unreachable!()
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
        _found: TokenKind,
        _span: Span,
        _expected: &[TokenKind],
    ) {
    }
    fn unexpected_eof(&mut self, _expected: &[TokenKind]) {}
}

/// A no-op set of callbacks.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nop;

impl Callbacks for Nop {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fail;

    impl Callbacks for Fail {
        fn unexpected_token(
            &mut self,
            found: TokenKind,
            span: Span,
            expected: &[TokenKind],
        ) {
            panic!(
                "Unexpected token, \"{}\" at {:?}. Expected {:?}",
                found, span, expected
            );
        }
        fn unexpected_eof(&mut self, expected: &[TokenKind]) {
            panic!("Unexpected EOF. Expected {:?}", expected);
        }
    }

    #[test]
    fn parse_a_comment_block() {
        let src = "; This is a comment\n";
        let mut parser = Parser::new_with_callbacks(src, Fail);

        let block = parser.next().unwrap();

        assert!(block.commands().is_empty());
        assert_eq!(block.comments().len(), 1);

        let comment = &block.comments()[0];
        assert_eq!(comment.body, "; This is a comment");
        assert_eq!(
            comment.span,
            Span {
                start: 0,
                end: src.len() - 1,
                source_line: 0
            }
        );
        assert_eq!(block.span(), comment.span);
    }

    #[test]
    fn read_a_line_number() {
        let mut parser = Parser::new_with_callbacks("N42", Fail);

        let block = parser.next().unwrap();

        assert_eq!(block.line_number(), Some(42));
        assert!(block.comments().is_empty());
        assert!(block.commands().is_empty());
    }

    #[test]
    fn read_a_g90() {
        let mut parser = Parser::new_with_callbacks("G90", Fail);

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
        let mut parser = Parser::new_with_callbacks("/N20 G90", Fail);

        let block = parser.next().unwrap();

        assert_eq!(block.line_number(), Some(20));
        assert!(block.comments().is_empty());
        assert!(block.deleted());

        assert_eq!(block.commands().len(), 1);
    }
}
