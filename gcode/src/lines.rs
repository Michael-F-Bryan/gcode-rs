use crate::{
    buffers::{Buffer, Buffers, CapacityError, DefaultBuffers},
    lexer::{Lexer, Token, TokenType},
    words::{Atom, Word, WordsOrComments},
    Comment, GCode, Mnemonic, Span,
};
use core::{iter::Peekable, marker::PhantomData};

/// A single line, possibly containing some [`Comment`]s or [`GCode`]s.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
pub struct Line<'input, B: Buffers<'input> = DefaultBuffers> {
    gcodes: B::Commands,
    comments: B::Comments,
    line_number: Option<Word>,
    span: Span,
}

impl<'input, B> Default for Line<'input, B>
where
    B: Buffers<'input>,
    B::Commands: Default,
    B::Comments: Default,
{
    fn default() -> Line<'input, B> {
        Line {
            gcodes: B::Commands::default(),
            comments: B::Comments::default(),
            line_number: None,
            span: Span::default(),
        }
    }
}

impl<'input, B: Buffers<'input>> Line<'input, B> {
    /// All [`GCode`]s in this line.
    pub fn gcodes(&self) -> &[GCode<B::Arguments>] { self.gcodes.as_slice() }

    /// All [`Comment`]s in this line.
    pub fn comments(&self) -> &[Comment<'input>] { self.comments.as_slice() }

    /// Try to add another [`GCode`] to the line.
    pub fn push_gcode(
        &mut self,
        gcode: GCode<B::Arguments>,
    ) -> Result<(), CapacityError<GCode<B::Arguments>>> {
        // Note: We need to make sure a failed push doesn't change our span
        let span = self.span.merge(gcode.span());
        self.gcodes.try_push(gcode)?;
        self.span = span;

        Ok(())
    }

    /// Try to add a [`Comment`] to the line.
    pub fn push_comment(
        &mut self,
        comment: Comment<'input>,
    ) -> Result<(), CapacityError<Comment<'input>>> {
        let span = self.span.merge(comment.span);
        self.comments.try_push(comment)?;
        self.span = span;
        Ok(())
    }

    /// Does the [`Line`] contain anything at all?
    pub fn is_empty(&self) -> bool {
        self.gcodes.as_slice().is_empty()
            && self.comments.as_slice().is_empty()
            && self.line_number().is_none()
    }

    /// Try to get the line number, if there was one.
    pub fn line_number(&self) -> Option<Word> { self.line_number }

    /// Set the [`Line::line_number()`].
    pub fn set_line_number<W: Into<Option<Word>>>(&mut self, line_number: W) {
        match line_number.into() {
            Some(n) => {
                self.span = self.span.merge(n.span);
                self.line_number = Some(n);
            },
            None => self.line_number = None,
        }
    }

    /// Get the [`Line`]'s position in its source text.
    pub fn span(&self) -> Span { self.span }
}

/// Callbacks used during the parsing process to indicate possible errors.
pub trait Callbacks {
    /// The parser encountered some text it wasn't able to make sense of.
    fn unknown_content(&mut self, _text: &str, _span: Span) {}

    /// The [`Buffers::Commands`] buffer had insufficient capacity when trying
    /// to add a [`GCode`].
    fn gcode_buffer_overflowed(
        &mut self,
        _mnemonic: Mnemonic,
        _major_number: u32,
        _minor_number: u32,
        _arguments: &[Word],
        _span: Span,
    ) {
    }

    /// The [`Buffers::Arguments`] buffer had insufficient capacity when trying
    /// to add a [`Word`].
    ///
    /// To aid in diagnostics, the caller is also given the [`GCode`]'s
    /// mnemonic and major/minor numbers.
    fn gcode_argument_buffer_overflowed(
        &mut self,
        _mnemonic: Mnemonic,
        _major_number: u32,
        _minor_number: u32,
        _argument: Word,
    ) {
    }

    /// A [`Comment`] was encountered, but there wasn't enough room in
    /// [`Buffers::Comments`].
    fn comment_buffer_overflow(&mut self, _comment: Comment<'_>) {}

    /// A line number was encountered when it wasn't expected.
    fn unexpected_line_number(&mut self, _line_number: f32, _span: Span) {}

    /// An argument was found, but the parser couldn't figure out which
    /// [`GCode`] it corresponds to.
    fn argument_without_a_command(
        &mut self,
        _letter: char,
        _value: f32,
        _span: Span,
    ) {
    }

    /// A [`Word`]'s number was encountered without an accompanying letter.
    fn number_without_a_letter(&mut self, _value: &str, _span: Span) {}

    /// A [`Word`]'s letter was encountered without an accompanying number.
    fn letter_without_a_number(&mut self, _value: &str, _span: Span) {}
}

impl<'a, C: Callbacks + ?Sized> Callbacks for &'a mut C {
    fn unknown_content(&mut self, text: &str, span: Span) {
        (*self).unknown_content(text, span);
    }

    fn gcode_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        arguments: &[Word],
        span: Span,
    ) {
        (*self).gcode_buffer_overflowed(
            mnemonic,
            major_number,
            minor_number,
            arguments,
            span,
        );
    }

    fn gcode_argument_buffer_overflowed(
        &mut self,
        mnemonic: Mnemonic,
        major_number: u32,
        minor_number: u32,
        argument: Word,
    ) {
        (*self).gcode_argument_buffer_overflowed(
            mnemonic,
            major_number,
            minor_number,
            argument,
        );
    }

    fn comment_buffer_overflow(&mut self, comment: Comment<'_>) {
        (*self).comment_buffer_overflow(comment);
    }

    fn unexpected_line_number(&mut self, line_number: f32, span: Span) {
        (*self).unexpected_line_number(line_number, span);
    }

    fn argument_without_a_command(
        &mut self,
        letter: char,
        value: f32,
        span: Span,
    ) {
        (*self).argument_without_a_command(letter, value, span);
    }

    fn number_without_a_letter(&mut self, value: &str, span: Span) {
        (*self).number_without_a_letter(value, span);
    }

    fn letter_without_a_number(&mut self, value: &str, span: Span) {
        (*self).letter_without_a_number(value, span);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct NopCallbacks;

impl Callbacks for NopCallbacks {}

/// Parse each [`Line`] in some text, ignoring any errors that may occur.
pub fn parse<'input>(
    src: &'input str,
) -> impl Iterator<Item = Line<'input>> + 'input {
    parse_with_callbacks(src, NopCallbacks)
}

/// Parse each line in some text, using the provided [`Callbacks`] when a parse
/// error occurs that we can recover from.
pub fn parse_with_callbacks<'input, C: Callbacks + 'input>(
    src: &'input str,
    callbacks: C,
) -> impl Iterator<Item = Line<'input>> + 'input {
    let tokens = Lexer::new(src);
    let atoms = WordsOrComments::new(tokens);
    Lines::new(atoms, callbacks)
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
    C: Callbacks,
    B: Buffers<'input>,
{
    fn new(atoms: I, callbacks: C) -> Self {
        Lines {
            atoms: atoms.peekable(),
            callbacks,
            last_gcode_type: None,
            _buffers: PhantomData,
        }
    }

    fn handle_line_number(
        &mut self,
        word: Word,
        line: &mut Line<'input, B>,
        temp_gcode: &Option<GCode<B::Arguments>>,
    ) {
        if line.gcodes().is_empty()
            && line.line_number().is_none()
            && temp_gcode.is_none()
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
        if let Some(mnemonic) = Mnemonic::for_letter(word.letter) {
            // we need to start another gcode. push the one we were building
            // onto the line so we can start working on the next one
            self.last_gcode_type = Some(word);
            if let Some(completed) = temp_gcode.take() {
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
        // we need a scratch space while processing arguments
        let mut temp_gcode = None;

        while let Some(next_span) = self.atoms.peek().map(|a| a.span()) {
            if !line.is_empty() && next_span.line != line.span.line {
                // we've started the next line
                break;
            }

            match self.atoms.next().expect("unreachable") {
                Atom::Unknown(token) => {
                    self.callbacks.unknown_content(token.value, token.span)
                },
                Atom::Comment(comment) => {
                    if let Err(e) = line.push_comment(comment) {
                        self.on_comment_push_error(e.0);
                    }
                },
                // line numbers are annoying, so handle them separately
                Atom::Word(word) if word.letter.to_ascii_lowercase() == 'n' => {
                    self.handle_line_number(word, &mut line, &temp_gcode);
                },
                Atom::Word(word) => {
                    self.handle_arg(word, &mut line, &mut temp_gcode)
                },
                Atom::BrokenWord(token) => self.handle_broken_word(token),
            }
        }

        if let Some(gcode) = temp_gcode {
            if let Err(e) = line.push_gcode(gcode) {
                self.on_gcode_push_error(e.0);
            }
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
        let unexpected_line_number = Default::default();
        let got: Vec<_> = parse_with_callbacks(
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

    #[test]
    fn multiple_commands_on_the_same_line() {
        let src = "G01 X5 G90 (comment) G91 M10\nG01";
        let got: Vec<_> = parse(src).collect();

        assert_eq!(got.len(), 2);
        let line = &got[0];
        assert_eq!(line.gcodes.len(), 4);
    }

    /// I wasn't sure if the `#[derive(Serialize)]` would work given we use
    /// `B::Comments`, which would borrow from the original source.
    #[test]
    #[cfg(feature = "serde-1")]
    fn you_can_actually_serialize_lines() {
        let src = "G01 X5 G90 (comment) G91 M10\nG01";
        let line = parse(src).next().unwrap();
        
        fn assert_serializable<S: serde::Serialize>(_: &S) {}
        fn assert_deserializable<'de, D: serde::Deserialize<'de>>() {}

        assert_serializable(&line);
        assert_deserializable::<Line<'_>>();
    }
}
