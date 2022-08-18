use crate::{
    buffers::{Buffers, DefaultBuffers},
    lexer::{Lexer, Token, TokenType},
    words::{Atom, Word, WordsOrComments},
    Callbacks, Comment, GCode, Line, Mnemonic, Nop,
};
use core::{iter::Peekable, marker::PhantomData};

/// Parse each [`GCode`] in some text, ignoring any errors that may occur or
/// [`Comment`]s that are found.
///
/// This function is probably what you are looking for if you just want to read
/// the [`GCode`] commands in a program. If more detailed information is needed,
/// have a look at [`full_parse_with_callbacks()`].
pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = GCode> + 'input {
    full_parse_with_callbacks(src, Nop).flat_map(|line| line.into_gcodes())
}

/// Parse each [`Line`] in some text, using the provided [`Callbacks`] when a
/// parse error occurs that we can recover from.
///
/// Unlike [`parse()`], this function will also give you access to any comments
/// and line numbers that are found, plus the location of the entire [`Line`]
/// in its source text.
pub fn full_parse_with_callbacks<'input, C: Callbacks + 'input>(
    src: &'input str,
    callbacks: C,
) -> impl Iterator<Item = Line<'input>> + 'input {
    let tokens = Lexer::new(src);
    let atoms = WordsOrComments::new(tokens);
    Lines::new(atoms, callbacks)
}

/// A parser for parsing g-code programs.
#[derive(Debug)]
pub struct Parser<'input, C, B = DefaultBuffers> {
    // Explicitly instantiate Lines so Parser's type parameters don't expose
    // internal details
    lines: Lines<'input, WordsOrComments<'input, Lexer<'input>>, C, B>,
}

impl<'input, C, B> Parser<'input, C, B> {
    /// Create a new [`Parser`] from some source text and a set of
    /// [`Callbacks`].
    pub fn new(src: &'input str, callbacks: C) -> Self {
        let tokens = Lexer::new(src);
        let atoms = WordsOrComments::new(tokens);
        let lines = Lines::new(atoms, callbacks);
        Parser { lines }
    }
}

impl<'input, B> From<&'input str> for Parser<'input, Nop, B> {
    fn from(src: &'input str) -> Self { Parser::new(src, Nop) }
}

impl<'input, C: Callbacks, B: Buffers<'input>> Iterator
    for Parser<'input, C, B>
{
    type Item = Line<'input, B>;

    fn next(&mut self) -> Option<Self::Item> { self.lines.next() }
}

#[derive(Debug)]
struct Lines<'input, I, C, B>
where
    I: Iterator<Item = Atom<'input>>,
{
    atoms: Peekable<I>,
    callbacks: C,
    last_gcode_type: Option<Word>,
    _buffers: PhantomData<B>,
}

impl<'input, I, C, B> Lines<'input, I, C, B>
where
    I: Iterator<Item = Atom<'input>>,
{
    fn new(atoms: I, callbacks: C) -> Self {
        Lines {
            atoms: atoms.peekable(),
            callbacks,
            last_gcode_type: None,
            _buffers: PhantomData,
        }
    }
}

impl<'input, I, C, B> Lines<'input, I, C, B>
where
    I: Iterator<Item = Atom<'input>>,
    C: Callbacks,
    B: Buffers<'input>,
{
    fn handle_line_number(
        &mut self,
        word: Word,
        line: &mut Line<'input, B>,
        has_temp_gcode: bool,
    ) {
        if line.gcodes().is_empty()
            && line.line_number().is_none()
            && !has_temp_gcode
        {
            line.set_line_number(word);
        } else {
            self.callbacks.unexpected_line_number(word.value, word.span);
        }
    }

    fn handle_arg(
        &mut self,
        word: Word,
        line: &mut Line<'input, B>,
        temp_gcode: &mut Option<GCode<B::Arguments>>,
    ) {
        // First, we check to see if the character is actually a new command.
        if let Some(mnemonic) = Mnemonic::for_letter(word.letter) {
            // We need to start another gcode.

            self.last_gcode_type = Some(word);

            if let Some(completed) = temp_gcode.take() {
                // We were already in progress building arguments for this code, and now we found
                // a new command that effectively ends the previous command.

                // Push the g-code we were building onto the line so we can start working on the next one.
                if let Err(e) = line.push_gcode(completed) {
                    self.on_gcode_push_error(e.0);
                }
            }

            *temp_gcode = Some(GCode::new_with_argument_buffer(
                mnemonic,
                word.value,
                word.span,
                B::Arguments::default(),
            ));
            
            return;
        }

        // we've got an argument, try adding it to the gcode we're building
        if let Some(temp) = temp_gcode {
            if let Err(e) = temp.push_argument(word) {
                self.on_arg_push_error(&temp, e.0);
            }
            return;
        }

        // we haven't already started building a gcode, maybe the author elided
        // the command ("G90") and wants to use the one from the last line?
        match self.last_gcode_type {
            Some(ty) => {
                let mut new_gcode = GCode::new_with_argument_buffer(
                    Mnemonic::for_letter(ty.letter).unwrap(),
                    ty.value,
                    ty.span,
                    B::Arguments::default(),
                );
                if let Err(e) = new_gcode.push_argument(word) {
                    self.on_arg_push_error(&new_gcode, e.0);
                }
                *temp_gcode = Some(new_gcode);
            },
            // oh well, you can't say we didn't try...
            None => {
                self.callbacks.argument_without_a_command(
                    word.letter,
                    word.value,
                    word.span,
                );
            },
        }
    }

    fn handle_broken_word(&mut self, token: Token<'_>) {
        if token.kind == TokenType::Letter {
            self.callbacks
                .letter_without_a_number(token.value, token.span);
        } else {
            self.callbacks
                .number_without_a_letter(token.value, token.span);
        }
    }

    fn on_arg_push_error(&mut self, gcode: &GCode<B::Arguments>, arg: Word) {
        self.callbacks.gcode_argument_buffer_overflowed(
            gcode.mnemonic(),
            gcode.major_number(),
            gcode.minor_number(),
            arg,
        );
    }

    fn on_comment_push_error(&mut self, comment: Comment<'_>) {
        self.callbacks.comment_buffer_overflow(comment);
    }

    fn on_gcode_push_error(&mut self, gcode: GCode<B::Arguments>) {
        self.callbacks.gcode_buffer_overflowed(
            gcode.mnemonic(),
            gcode.major_number(),
            gcode.minor_number(),
            gcode.arguments(),
            gcode.span(),
        );
    }

}

impl<'input, I, C, B> Iterator for Lines<'input, I, C, B>
where
    I: Iterator<Item = Atom<'input>> + 'input,
    C: Callbacks,
    B: Buffers<'input>,
{
    type Item = Line<'input, B>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Line::default();
        // we need a scratch space for the gcode we're in the middle of
        // constructing
        let mut temp_gcode = None;

        if let None = self.atoms.peek() {
            // There is nothing left in the file. :sad-face:
            // This ends the parser's work.
            return None;
        }

        while let Some(atom) = self.atoms.next() {
            match atom {
                Atom::Unknown(token) => {
                    self.callbacks.unknown_content(token.value, token.span)
                },
                Atom::Comment(comment) => {
                    if let Err(e) = line.push_comment(comment) {
                        self.on_comment_push_error(e.0);
                    }
                },
                Atom::Newline(_) => {
                    if !line.is_empty() || temp_gcode.is_some() {
                        // Newline ends the current command if there was something to parse.
                        break;
                    }
                    // Otherwise, the g-code had an empty line and we can ignore it.
                },
                // line numbers are annoying, so handle them separately
                Atom::Word(word) if word.letter.to_ascii_lowercase() == 'n' => {
                    self.handle_line_number(
                        word,
                        &mut line,
                        temp_gcode.is_some(),
                    );
                },
                Atom::Word(word) => {
                    self.handle_arg(word, &mut line, &mut temp_gcode)
                },
                Atom::BrokenWord(token) => self.handle_broken_word(token),
            }
        }

        if let Some(gcode) = temp_gcode.take() {
            if let Err(e) = line.push_gcode(gcode) {
                self.on_gcode_push_error(e.0);
            }
        }

        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;
    use arrayvec::ArrayVec;
    use std::{prelude::v1::*, sync::Mutex};

    #[derive(Debug)]
    struct MockCallbacks<'a> {
        unexpected_line_number: &'a Mutex<Vec<(f32, Span)>>,
    }

    impl<'a> Callbacks for MockCallbacks<'a> {
        fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
            self.unexpected_line_number
                .lock()
                .unwrap()
                .push((line_number, span));
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    enum BigBuffers {}

    impl<'input> Buffers<'input> for BigBuffers {
        type Arguments = ArrayVec<[Word; 16]>;
        type Commands = ArrayVec<[GCode<Self::Arguments>; 16]>;
        type Comments = ArrayVec<[Comment<'input>; 16]>;
    }

    fn parse(
        src: &str,
    ) -> Lines<'_, impl Iterator<Item = Atom<'_>>, Nop, BigBuffers> {
        let tokens = Lexer::new(src);
        let atoms = WordsOrComments::new(tokens);
        Lines::new(atoms, Nop)
    }

    #[test]
    fn we_can_parse_a_comment() {
        let src = "(this is a comment)";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.comments().len(), 1);
        assert_eq!(line.gcodes().len(), 0);
        assert_eq!(line.span(), Span::new(0, src.len(), 0));
    }

    #[test]
    fn line_numbers() {
        let src = "N42";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.comments().len(), 0);
        assert_eq!(line.gcodes().len(), 0);
        let span = Span::new(0, src.len(), 0);
        assert_eq!(
            line.line_number(),
            Some(Word {
                letter: 'N',
                value: 42.0,
                span
            })
        );
        assert_eq!(line.span(), span);
    }

    #[test]
    fn line_numbers_after_the_start_are_an_error() {
        let src = "G90 N42";
        let unexpected_line_number = Default::default();
        let got: Vec<_> = full_parse_with_callbacks(
            src,
            MockCallbacks {
                unexpected_line_number: &unexpected_line_number,
            },
        )
        .collect();

        assert_eq!(got.len(), 1);
        assert!(got[0].line_number().is_none());
        let unexpected_line_number = unexpected_line_number.lock().unwrap();
        assert_eq!(unexpected_line_number.len(), 1);
        assert_eq!(unexpected_line_number[0].0, 42.0);
    }

    #[test]
    fn parse_g90() {
        let src = "G90";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.gcodes().len(), 1);
        let g90 = &line.gcodes()[0];
        assert_eq!(g90.major_number(), 90);
        assert_eq!(g90.minor_number(), 0);
        assert_eq!(g90.arguments().len(), 0);
    }

    #[test]
    fn parse_command_with_arguments() {
        let src = "G01X5 Y-20";
        let should_be =
            GCode::new(Mnemonic::General, 1.0, Span::new(0, src.len(), 0))
                .with_argument(Word {
                    letter: 'X',
                    value: 5.0,
                    span: Span::new(3, 5, 0),
                })
                .with_argument(Word {
                    letter: 'Y',
                    value: -20.0,
                    span: Span::new(6, 10, 0),
                });

        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.gcodes().len(), 1);
        let g01 = &line.gcodes()[0];
        assert_eq!(g01, &should_be);
    }

    #[test]
    fn multiple_commands_on_the_same_line() {
        let src = "G01 X5 G90 (comment) G91 M10\nG01";

        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 2);
        assert_eq!(got[0].gcodes().len(), 4);
        assert_eq!(got[0].comments().len(), 1);
        assert_eq!(got[1].gcodes().len(), 1);
    }

    #[test]
    fn funny_bug_in_crate_example() {
        let src = "G90 \n G01 X50.0 Y-10";
        let expected = vec![
            GCode::new(Mnemonic::General, 90.0, Span::PLACEHOLDER),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 50.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', -10.0, Span::PLACEHOLDER)),
        ];

        let got: Vec<_> = crate::parse(src).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn implicit_command_after_newline() {
        let src = "M3\nG01 X1.0 Y2.0\nX3.0 Y4.0";
        let expected = vec![
            GCode::new(Mnemonic::Miscellaneous, 3.0, Span::PLACEHOLDER),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 1.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 2.0, Span::PLACEHOLDER)),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 3.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 4.0, Span::PLACEHOLDER)),
        ];

        let got: Vec<_> = crate::parse(src).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn implicit_command_standalone() {
        let src = "G01 X1.0 Y2.0\nX3.0 Y4.0";
        let expected = vec![
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 1.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 2.0, Span::PLACEHOLDER)),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 3.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 4.0, Span::PLACEHOLDER)),
        ];

        let got: Vec<_> = crate::parse(src).collect();
        assert_eq!(got, expected);
    }

    #[test]
    // This test focuses on the G90 and M7 on the same line.
    fn implicit_command_two_commands_on_line() {
        let src = "G90 M7\nG01 X1.0 Y2.0\nX3.0 Y4.0";
        let expected = vec![
            GCode::new(Mnemonic::General, 90.0, Span::PLACEHOLDER),
            GCode::new(Mnemonic::Miscellaneous, 7.0, Span::PLACEHOLDER),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 1.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 2.0, Span::PLACEHOLDER)),
            GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                .with_argument(Word::new('X', 3.0, Span::PLACEHOLDER))
                .with_argument(Word::new('Y', 4.0, Span::PLACEHOLDER)),
        ];

        let got: Vec<_> = crate::parse(src).collect();
        assert_eq!(got, expected);
    }
}
