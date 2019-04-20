cfg_if! {
    if #[cfg(feature = "std")] {
        use std::cmp;
        use std::fmt::{self, Display, Formatter};
        use std::borrow::Cow;

        type Comments<'input> = Vec<Comment<'input>>;
        type Arguments = Vec<Argument>;
        type Commands = Vec<Gcode>;
    } else {
        use core::cmp;
        #[allow(unused_imports)]
        use libm::F32Ext;
        use arrayvec::ArrayVec;
        use core::fmt::{self, Display, Formatter};

        type Comments<'input> = ArrayVec<[Comment<'input>; Block::MAX_COMMENT_COUNT]>;
        type Arguments = ArrayVec<[Argument; Gcode::MAX_ARGUMENT_COUNT]>;
        type Commands = ArrayVec<[Gcode; Block::MAX_COMMAND_COUNT]>;
    }
}

/// The location of something within a string.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Span {
    /// The byte index corresponding to the beginning of this `Span`.
    pub start: usize,
    /// The byte index *one past* the end of the selection.
    pub end: usize,
    /// Which line does this `Span` start on?
    pub source_line: usize,
}

impl Span {
    /// Create a new `Span`.
    pub fn new(start: usize, end: usize, source_line: usize) -> Span {
        Span {
            start,
            end,
            source_line,
        }
    }

    /// Get the placeholder `Span`, representing a location which hasn't been
    /// resolved yet or doesn't correspond to a location (e.g. generated code).
    pub fn placeholder() -> Span {
        Span {
            start: usize::max_value(),
            end: usize::max_value(),
            source_line: usize::max_value(),
        }
    }

    /// Is this the placeholder `Span`?
    pub fn is_placeholder(&self) -> bool {
        *self == Span::placeholder()
    }

    /// Based on this `Span`, get the sub-string it corresponds to.
    pub fn text_from_source<'input>(
        &self,
        src: &'input str,
    ) -> Option<&'input str> {
        src.get(self.start..self.end)
    }

    /// Merge this `Span` with another one.
    pub fn merge(&self, other: Span) -> Span {
        if self.is_placeholder() {
            other
        } else if other.is_placeholder() {
            *self
        } else {
            Span {
                start: cmp::min(self.start, other.start),
                end: cmp::max(self.end, other.end),
                source_line: cmp::min(self.source_line, other.source_line),
            }
        }
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::placeholder()
    }
}

/// The various token types that make up a gcode program.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum TokenKind {
    Letter,
    Number,
    Comment,
    Newline,
    ForwardSlash,
    Percent,
    Garbage,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            TokenKind::Letter => "letter".fmt(f),
            TokenKind::Number => "number".fmt(f),
            TokenKind::Comment => "comment".fmt(f),
            TokenKind::Newline => "newline".fmt(f),
            TokenKind::ForwardSlash => "forward-slash".fmt(f),
            TokenKind::Percent => "percent".fmt(f),
            TokenKind::Garbage => "garbage".fmt(f),
        }
    }
}

/// A block containing `Gcode` commands and/or comments.
#[derive(Debug, Clone, PartialEq)]
pub struct Block<'input> {
    line_number: Option<usize>,
    deleted: bool,
    span: Span,
    commands: Commands,
    comments: Comments<'input>,
}

impl<'input> Block<'input> {
    /// The maximum number of commands which can be in a [`Block`] when compiled
    /// *without* the `std` feature.
    pub const MAX_COMMAND_COUNT: usize = 10;
    /// The maximum number of [`Comment`]s which can be in a [`Block`] when
    /// compiled *without* the `std` feature.
    pub const MAX_COMMENT_COUNT: usize = 10;

    pub(crate) fn empty() -> Block<'input> {
        Block {
            commands: Default::default(),
            comments: Default::default(),
            deleted: false,
            line_number: None,
            span: Span::placeholder(),
        }
    }

    /// Is this block empty?
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
            && self.comments.is_empty()
            && self.line_number.is_none()
    }

    /// Convert this `Block` into a stream of `Gcode` commands.
    pub fn into_commands(self) -> impl Iterator<Item = Gcode> {
        self.commands.into_iter()
    }

    /// The `Gcode`s in this block.
    pub fn commands(&self) -> &[Gcode] {
        &self.commands
    }

    /// Get a mutable reference to the `Gcode`s in this block.
    pub fn commands_mut(&mut self) -> &mut [Gcode] {
        &mut self.commands
    }

    /// The comments in this block.
    pub fn comments(&self) -> &[Comment<'input>] {
        &self.comments
    }

    /// The block's location.
    pub fn span(&self) -> Span {
        self.span
    }

    /// The block's line number (e.g. `N42`), if provided.
    pub fn line_number(&self) -> Option<usize> {
        self.line_number
    }

    /// Set the `Block`'s line number.
    pub fn with_line_number(&mut self, number: usize, span: Span) -> &mut Self {
        self.merge_span(span);
        self.line_number = Some(number);
        self
    }

    /// Does this block start with a *block delete*?
    pub fn deleted(&self) -> bool {
        self.deleted
    }

    /// Set the *block delete* flag.
    pub fn delete(&mut self, delete: bool) {
        self.deleted = delete;
    }

    /// Add a `Comment` to the `Block`.
    pub fn push_comment(&mut self, comment: Comment<'input>) {
        self.merge_span(comment.span);
        self.comments.push(comment);
    }

    /// Add a `Gcode` to the `Block`.
    pub fn push_command(&mut self, command: Gcode) {
        self.merge_span(command.span);
        self.commands.push(command);
    }

    fn merge_span(&mut self, span: Span) {
        if self.span.is_placeholder() {
            self.span = span;
        } else {
            self.span = self.span.merge(span);
        }
    }
}

/// A single gcode command.
#[derive(Debug, Clone, PartialEq)]
pub struct Gcode {
    line_number: Option<usize>,
    mnemonic: Mnemonic,
    number: f32,
    span: Span,
    arguments: Arguments,
}

impl Gcode {
    /// The maximum number of arguments in a single [`Gcode`] command when
    /// compiled *without* the `std` feature.
    pub const MAX_ARGUMENT_COUNT: usize = 10;

    /// Create a new `Gcode`.
    ///
    /// # Panics
    ///
    /// A negative `number` is invalid.
    pub fn new(mnemonic: Mnemonic, number: f32) -> Gcode {
        debug_assert!(number >= 0.0, "The number should always be positive");
        Gcode {
            mnemonic,
            number,
            line_number: None,
            arguments: Default::default(),
            span: Default::default(),
        }
    }

    /// The general command category.
    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }

    /// Get the command's major number.
    pub fn major_number(&self) -> usize {
        debug_assert!(
            self.number >= 0.0,
            "The number should always be positive"
        );
        self.number.trunc() as usize
    }

    /// Get the command's minor number, of there is one.
    pub fn minor_number(&self) -> Option<usize> {
        debug_assert!(
            self.number >= 0.0,
            "The number should always be positive"
        );
        let digit = (self.number.fract() / 0.1).round();

        if digit == 0.0 {
            None
        } else {
            Some(digit as usize)
        }
    }

    /// Access any arguments attached to this `Gcode`.
    pub fn args(&self) -> &[Argument] {
        &self.arguments
    }

    /// Mutably access the arguments attached to this `Gcode`.
    pub fn args_mut(&mut self) -> &mut [Argument] {
        &mut self.arguments
    }

    /// Set the line number.
    pub fn with_line_number(mut self, number: usize) -> Self {
        self.line_number = Some(number);
        self
    }

    /// Set the minor number.
    pub fn with_minor_number(mut self, number: usize) -> Self {
        debug_assert!(number < 10);
        self.number = self.number.trunc() + number as f32 / 10.0;
        self
    }

    /// Add an argument to the [`Gcode`], removing any previous arguments with
    /// the same letter.
    pub fn with_argument(mut self, arg: Argument) -> Self {
        while let Some(_) = self.remove_argument(arg.letter) {}

        self.push_argument(arg);
        self
    }

    /// Set the `Gcode`'s `Span`.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Add an argument to the `Gcode`.
    pub fn push_argument(&mut self, arg: Argument) {
        if !arg.span.is_placeholder() {
            self.span = self.span.merge(arg.span);
        }
        self.arguments.push(arg);
    }

    /// Remove the first argument with the specified `letter`.
    pub fn remove_argument(&mut self, letter: char) -> Option<Argument> {
        if let Some(ix) =
            self.arguments.iter().position(|arg| arg.letter == letter)
        {
            let removed = self.arguments.remove(ix);
            Some(removed)
        } else {
            None
        }
    }

    /// Get the `Gcode`'s location.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the `Gcode`'s line number, if one was provided.
    pub fn line_number(&self) -> Option<usize> {
        self.line_number
    }

    /// Get the value for a particular argument letter, if there is one.
    pub fn value_for(&self, letter: char) -> Option<f32> {
        let letter = letter.to_ascii_lowercase();

        self.args()
            .iter()
            .find(|word| word.letter.to_ascii_lowercase() == letter)
            .map(|word| word.value)
    }
}

/// A command argument, the `X50.0` in `G01 X50.0`.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Argument {
    /// The `X` in `X50.0`.
    pub letter: char,
    /// The `50.0` in `X50.0`.
    pub value: f32,
    /// Where the `Argument` lies in the source text.
    pub span: Span,
}

impl Argument {
    /// Create a new `Argument`.
    pub fn new(letter: char, value: f32, span: Span) -> Argument {
        Argument {
            letter,
            value,
            span,
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.value)
    }
}

/// A comment.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment<'input> {
    #[cfg(feature = "std")]
    body: Cow<'input, str>,
    #[cfg(not(feature = "std"))]
    body: &'input str,
    /// Where the `Comment` is placed within the source text.
    pub span: Span,
}

cfg_if! {
    if #[cfg(feature = "std")] {
        impl<'input> Comment<'input> {
            /// Create a new `Comment`.
            pub fn new(body: &'input str, span: Span) -> Comment<'input> {
                Comment::new_cow(body, span)
            }

            /// Get the `Comment`'s content.
            pub fn body(&self) -> &str {
                &*self.body
            }

            /// Create a new `Comment`.
            pub fn new_cow<S: Into<Cow<'input, str>>>(
                body: S,
                span: Span,
            ) -> Comment<'input> {
                Comment {
                    body: body.into(),
                    span,
                }
            }
        }
    } else {
        impl<'input> Comment<'input> {
            /// Create a new `Comment`.
            pub fn new(body: &'input str, span: Span) -> Comment<'input> {
                Comment { body, span }
            }

            /// Get the `Comment`'s content.
            pub fn body(&self) -> &str {
                self.body
            }
        }
    }
}

/// The general category a `Gcode` belongs to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Mnemonic {
    /// A *general* G-code, typically things like move commands or used to
    /// change the machine's state.
    General,
    /// A miscellaneous command.
    Miscellaneous,
    /// The program number.
    ProgramNumber,
    /// A tool change.
    ToolChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_extract_major_and_minor_number() {
        let input = vec![
            (1.0, 1, None),
            (1.1, 1, Some(1)),
            (1.2, 1, Some(2)),
            (1.3, 1, Some(3)),
            (1.4, 1, Some(4)),
            (1.5, 1, Some(5)),
            (1.6, 1, Some(6)),
            (2.7, 2, Some(7)),
        ];

        for (number, major, minor) in input {
            let g = Gcode::new(Mnemonic::General, number);

            assert_eq!(g.major_number(), major);
            assert_eq!(g.minor_number(), minor);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn create_a_cow_comment() {
        let span = Span::new(0, 1, 0);
        let comment = Comment::new("blah", span);

        assert_eq!(comment.body(), "blah");
        assert_eq!(comment.span, span);

        let cow_comment = Comment::new_cow(String::from("blah"), span);

        assert_eq!(comment, cow_comment);
    }
}
