use lexer::{Lexer, Token};
#[cfg(not(feature = "std"))]
use libm::F32Ext;
use types::{Argument, Block, Comment, Gcode, Mnemonic, Span, TokenKind};

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
        let _ = self.chomp(TokenKind::Newline, |c| block.push_comment(c));

        if block.is_empty() {
            None
        } else {
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
        match self.parse_word(|c| block.push_comment(c)) {
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
        mut comments: impl FnMut(Comment<'input>),
    ) -> Option<Argument> {
        let (tok, letter_span) =
            self.chomp(TokenKind::Letter, &mut comments)?;
        let letter = tok.unwrap_letter();

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
        mut comments: impl FnMut(Comment<'input>),
    ) {
        let mut overall_span = Span::placeholder();

        while let Some(kind) = self.next_kind() {
            if kind == TokenKind::Newline {
                break;
            }

            let (tok, span) = self.lexer.next().expect("We aren't at the EOF");
            overall_span = overall_span.merge(span);

            if let Token::Comment(body) = tok {
                comments(Comment::new(body, span));
            }
        }

        #[cfg(test)]
        println!("Overall span: {:?}", overall_span);

        self.callbacks.mangled_input(
            overall_span
                .text_from_source(self.src)
                .expect("Always within bounds"),
            overall_span,
        );
    }

    fn parse_command(
        &mut self,
        mut comments: impl FnMut(Comment<'input>),
    ) -> Option<Gcode> {
        let Argument {
            letter,
            span,
            value,
        } = self.parse_word(&mut comments)?;

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

        let mut cmd = Gcode::new(mnemonic, value);
        cmd.with_span(span);

        while self.next_is_argument() {
            match self.parse_word(&mut comments) {
                Some(arg) => {
                    cmd.with_argument(arg);
                }
                None => {
                    // TODO: Signal an error to the callbacks
                    unimplemented!("Error parsing {:?} at {:?}", cmd, span);
                }
            }
        }

        Some(cmd)
    }

    fn next_is_argument(&self) -> bool {
        if let Some(letter) = self.next_letter() {
            match letter {
                'G' | 'g' | 'M' | 'm' | 'T' | 't' | 'O' | 'o' => false,
                _ => true,
            }
        } else {
            false
        }
    }

    fn next_starts_a_command(&self) -> bool {
        if let Some(letter) = self.next_letter() {
            match letter.to_ascii_lowercase() {
                'g' | 'm' | 't' | 'o' => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn next_letter(&self) -> Option<char> {
        for (tok, _) in self.lexer.clone() {
            match tok {
                Token::Letter(l) => return Some(l),
                Token::Comment(_) => continue,
                _ => return None,
            }
        }

        None
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
                comments(Comment::new(body, span));
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

/// Callback functions the `Parser` can use to notify the user of errors
/// encountered while parsing.
pub trait Callbacks {
    fn unexpected_token(
        &mut self,
        _found: TokenKind,
        _span: Span,
        _expected: &[TokenKind],
    ) {
    }
    fn unexpected_eof(&mut self, _expected: &[TokenKind]) {}
    fn mangled_input(&mut self, input: &str, span: Span) {}
}

impl<'a, C: Callbacks> Callbacks for &'a mut C {
    fn unexpected_token(
        &mut self,
        found: TokenKind,
        span: Span,
        expected: &[TokenKind],
    ) {
        (**self).unexpected_token(found, span, expected);
    }
    fn unexpected_eof(&mut self, expected: &[TokenKind]) {
        (**self).unexpected_eof(expected);
    }
    fn mangled_input(&mut self, input: &str, span: Span) {
        (**self).mangled_input(input, span);
    }
}

/// A no-op set of callbacks.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nop;

impl Callbacks for Nop {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::v1::*;

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
        fn mangled_input(&mut self, input: &str, span: Span) {
            panic!("Mangled input at {:?}, {:?}", span, input);
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
        assert_eq!(comment.body(), "; This is a comment");
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

    #[test]
    fn parse_a_command_with_arguments() {
        let mut parser = Parser::new_with_callbacks("G00 X50.0 Y-20.5", Fail);

        let block = parser.next().unwrap();

        assert!(block.comments().is_empty());
        assert_eq!(block.commands().len(), 1);
        let got = &block.commands()[0];

        assert_eq!(got.major_number(), 0);
        assert_eq!(got.value_for('X').unwrap(), 50.0);
        assert_eq!(got.value_for('x').unwrap(), 50.0);
        assert_eq!(got.value_for('Y').unwrap(), -20.5);
    }

    #[test]
    fn parse_multiple_commands() {
        let src = "N42 G00 Z-0.5 G02 X5 I100.0 ; Some comment\n";

        let got: Vec<_> = Parser::new_with_callbacks(src, Fail).collect();
        assert_eq!(got.len(), 1);
        let block = &got[0];

        assert_eq!(block.line_number(), Some(42));
        let commands = block.commands();

        let g00 = &commands[0];
        assert_eq!(g00.major_number(), 0);
        assert_eq!(g00.args().len(), 1);
        assert_eq!(g00.value_for('z'), Some(-0.5));

        let g02 = &commands[1];
        assert_eq!(g02.major_number(), 2);
        assert_eq!(g02.args().len(), 2);
        assert_eq!(g02.value_for('X'), Some(5.0));
        assert_eq!(g02.value_for('I'), Some(100.0));

        assert_eq!(block.comments().len(), 1);
        let comment = &block.comments()[0];
        assert_eq!(comment.body(), "; Some comment");
    }

    #[derive(Debug, Default)]
    struct GarbageCollector {
        garbage: Vec<(String, Span)>,
    }

    impl Callbacks for GarbageCollector {
        fn mangled_input(&mut self, input: &str, span: Span) {
            self.garbage.push((input.to_string(), span));
        }
    }

    #[test]
    fn skip_erroneous_sections() {
        let src = "N42 G42 $Oops... G90 (retain comments) P5";

        let mut gc = GarbageCollector::default();
        let got: Vec<_> = Parser::new_with_callbacks(src, &mut gc).collect();

        assert_eq!(got.len(), 1);

        let first_block = &got[0];
        let commands = first_block.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(first_block.comments().len(), 1);

        let g42 = &commands[0];
        assert_eq!(g42.major_number(), 42);
        assert_eq!(g42.span(), Span::new(4, 8, 0));
        assert!(g42.args().is_empty());

        assert_eq!(gc.garbage.len(), 1);
        let (garbage, span) = gc.garbage[0].clone();
        assert_eq!(garbage, &src[8..]);
        assert_eq!(span, Span::new(8, src.len(), 0));
    }
}
