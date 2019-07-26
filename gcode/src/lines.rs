use crate::lexer::Lexer;
use crate::words::{Atom, Word, WordsOrComments};
use crate::{Comment, GCode, Mnemonic, Span};
use core::iter::Peekable;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        type Commands = Vec<GCode>;
        type Comments<'input> = Vec<Comment<'input>>;
    } else {
        type Commands = ArrayVec<[GCode; MAX_COMMAND_LEN]>;
        type Comments<'input> = ArrayVec<[Comment<'input>; MAX_COMMENT_LEN]>;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "large_buffers")] {
        /// The maximum number of [`GCode`]s when compiled without the `std`
        /// feature.
        ///
        pub const MAX_COMMAND_LEN: usize = 2;
        /// The maximum number of [`Comment`]s when compiled without the `std`
        /// feature.
        ///
        pub const MAX_COMMENT_LEN: usize = 1;
    } else {
        /// The maximum number of [`GCode`]s when compiled without the `std`
        /// feature.
        ///
        pub const MAX_COMMAND_LEN: usize = 2;
        /// The maximum number of [`Comment`]s when compiled without the `std`
        /// feature.
        ///
        pub const MAX_COMMENT_LEN: usize = 1;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Line<'input> {
    gcodes: Commands,
    comments: Comments<'input>,
    line_number: Option<Word>,
    span: Span,
}

impl<'input> Line<'input> {
    pub fn gcodes(&self) -> &[GCode] {
        &self.gcodes
    }

    pub fn comments(&self) -> &[Comment<'input>] {
        &self.comments
    }

    pub fn push_gcode(&mut self, gcode: GCode) {
        self.span = self.span.merge(gcode.span());
        self.gcodes.push(gcode);
    }

    pub fn push_comment(&mut self, comment: Comment<'input>) {
        self.span = self.span.merge(comment.span);
        self.comments.push(comment);
    }

    pub fn is_empty(&self) -> bool {
        self.gcodes.is_empty() && self.comments.is_empty() && self.line_number().is_none()
    }

    pub fn line_number(&self) -> Option<Word> {
        self.line_number
    }

    pub fn set_line_number<W: Into<Option<Word>>>(&mut self, line_number: W) {
        match line_number.into() {
            Some(n) => {
                self.span = self.span.merge(n.span);
                self.line_number = Some(n);
            }
            None => self.line_number = None,
        }
    }
}

pub trait Callbacks {
    fn unknown_content(&mut self, _text: &str, _span: Span) {}
    fn gcode_buffer_overflowed(&mut self, _gcode: GCode) {}
    fn unexpected_line_number(&mut self, _line_number: f32, _span: Span) {}
    fn argument_without_a_command(&mut self, _letter: char, _value: f32, _span: Span) {}
}

impl<'a, C: Callbacks> Callbacks for &'a mut C {
    fn unknown_content(&mut self, text: &str, span: Span) {
        (*self).unknown_content(text, span);
    }

    fn gcode_buffer_overflowed(&mut self, gcode: GCode) {
        (*self).gcode_buffer_overflowed(gcode);
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        (*self).unexpected_line_number(line_number, span);
    }

    fn argument_without_a_command(&mut self, letter: char, value: f32, span: Span) {
        (*self).argument_without_a_command(letter, value, span);
    }
}

struct NopCallbacks;

impl Callbacks for NopCallbacks {}

pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Line<'input>> + 'input {
    parse_with_callbacks(src, NopCallbacks)
}

pub fn parse_with_callbacks<'input, C: Callbacks + 'input>(
    src: &'input str,
    callbacks: C,
) -> impl Iterator<Item = Line<'input>> + 'input {
    let tokens = Lexer::new(src);
    let atoms = WordsOrComments::new(tokens);
    Lines::new(atoms, callbacks)
}

struct Lines<'input, I, C>
where
    I: Iterator<Item = Atom<'input>>,
{
    atoms: Peekable<I>,
    callbacks: C,
    last_gcode_type: Option<Word>,
}

impl<'input, I, C> Lines<'input, I, C>
where
    I: Iterator<Item = Atom<'input>>,
    C: Callbacks,
{
    fn new(atoms: I, callbacks: C) -> Self {
        Lines {
            atoms: atoms.peekable(),
            callbacks,
            last_gcode_type: None,
        }
    }

    fn handle_line_number(&mut self, word: Word, line: &mut Line<'_>, temp_gcode: &Option<GCode>) {
        if line.gcodes().is_empty() && line.line_number().is_none() && temp_gcode.is_none() {
            line.set_line_number(word);
        } else {
            self.callbacks.unexpected_line_number(word.value, word.span);
        }
    }

    fn handle_arg(&mut self, word: Word, line: &mut Line<'_>, temp_gcode: &mut Option<GCode>) {
        if let Some(mnemonic) = Mnemonic::for_letter(word.letter) {
            // we need to start another gcode. push the one we were building
            // onto the line so we can start working on the next one
            self.last_gcode_type = Some(word);
            if let Some(completed) = temp_gcode.take() {
                line.push_gcode(completed);
            }
            *temp_gcode = Some(GCode::new(mnemonic, word.value, word.span));
            return;
        }

        // we've got an argument, try adding it to the gcode we're building
        if let Some(temp) = temp_gcode {
            temp.push_argument(word);
            return;
        }

        // we haven't already started building a gcode, maybe the author elided
        // the command ("G90") and wants to use the one from the last line?
        match self.last_gcode_type {
            Some(ty) => {
                let mut new_gcode =
                    GCode::new(Mnemonic::for_letter(ty.letter).unwrap(), ty.value, ty.span);
                new_gcode.push_argument(word);
                *temp_gcode = Some(new_gcode);
            }
            // oh well, you can't say we didn't try...
            None => {
                self.callbacks
                    .argument_without_a_command(word.letter, word.value, word.span);
            }
        }
    }
}

impl<'input, I, C> Iterator for Lines<'input, I, C>
where
    I: Iterator<Item = Atom<'input>> + 'input,
    C: Callbacks,
{
    type Item = Line<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Line::default();
        // we need a scratch space while processing arguments
        let mut temp_gcode = None;

        while let Some(next_span) = self.atoms.peek().map(|a| a.span()) {
            if !line.is_empty() && next_span.line != line.span.line {
                // we've started the next line
                return Some(line);
            }

            match self.atoms.next().expect("unreachable") {
                Atom::Unknown(token) => self.callbacks.unknown_content(token.value, token.span),
                Atom::Comment(comment) => line.push_comment(comment),
                // line numbers are annoying, so handle them separately
                Atom::Word(word) if word.letter.to_ascii_lowercase() == 'n' => {
                    self.handle_line_number(word, &mut line, &temp_gcode);
                }
                Atom::Word(word) => self.handle_arg(word, &mut line, &mut temp_gcode),
                _ => unimplemented!(),
            }
        }

        if let Some(gcode) = temp_gcode {
            line.push_gcode(gcode);
        }

        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct MockCallbacks {
        unexpected_line_number: Vec<(f32, Span)>,
    }

    impl Callbacks for MockCallbacks {
        fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
            self.unexpected_line_number.push((line_number, span));
        }
    }

    #[test]
    fn we_can_parse_a_comment() {
        let src = "(this is a comment)";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.comments().len(), 1);
        assert_eq!(line.gcodes().len(), 0);
        assert_eq!(line.span, Span::new(0, src.len(), 0));
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
        assert_eq!(line.span, span);
    }

    #[test]
    fn line_numbers_after_the_start_are_an_error() {
        let src = "G90 N42";
        let mut cb = MockCallbacks::default();
        let got: Vec<_> = parse_with_callbacks(src, &mut cb).collect();

        assert_eq!(got.len(), 1);
        assert!(got[0].line_number().is_none());
        assert_eq!(cb.unexpected_line_number.len(), 1);
        assert_eq!(cb.unexpected_line_number[0].0, 42.0);
    }

    #[test]
    fn parse_g90() {
        let src = "G90";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.gcodes.len(), 1);
        let g90 = &line.gcodes()[0];
        assert_eq!(g90.major_number(), 90);
        assert_eq!(g90.minor_number(), 0);
        assert_eq!(g90.arguments().len(), 0);
    }

    #[test]
    fn parse_command_with_arguments() {
        let src = "G01X5 Y-20";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 1);
        let line = &got[0];
        assert_eq!(line.gcodes.len(), 1);
        let g01 = &line.gcodes()[0];
        assert_eq!(g01.major_number(), 1);
        assert_eq!(g01.minor_number(), 0);
        let should_be = vec![
            Word {
                letter: 'X',
                value: 5.0,
                span: Span::new(3, 5, 0),
            },
            Word {
                letter: 'Y',
                value: -20.0,
                span: Span::new(6, 10, 0),
            },
        ];
        assert_eq!(g01.arguments(), should_be.as_slice());
    }
}
