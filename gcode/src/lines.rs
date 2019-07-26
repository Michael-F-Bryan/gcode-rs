use crate::lexer::Lexer;
use crate::words::{Atom, Word, WordsOrComments};
use crate::{Comment, GCode, Span};

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
    commands: Commands,
    comments: Comments<'input>,
}

impl<'input> Line<'input> {
    pub fn commands(&self) -> &[GCode] {
        &self.commands
    }

    pub fn comments(&self) -> &[Comment<'input>] {
        &self.comments
    }
}

pub trait Callbacks {
    fn unknown_content(&mut self, _text: &str, _span: Span) {}
    fn gcode_buffer_overflowed(&mut self, _gcode: GCode) {}
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

struct Lines<I, C> {
    atoms: I,
    callbacks: C,
    last_gcode_type: Option<Word>,
}

impl<'input, I, C> Lines<I, C>
where
    I: Iterator<Item = Atom<'input>>,
    C: Callbacks,
{
    fn new(atoms: I, callbacks: C) -> Self {
        Lines {
            atoms,
            callbacks,
            last_gcode_type: None,
        }
    }
}

impl<'input, I, C> Iterator for Lines<I, C>
where
    I: Iterator<Item = Atom<'input>> + 'input,
    C: Callbacks,
{
    type Item = Line<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
