cfg_if! {
    if #[cfg(feature = "std")] {
        use std::cmp;
        use std::fmt::{self, Display, Formatter};

    } else {
        use core::cmp;
        #[allow(unused_imports)]
        use libm::F32Ext;
        use arrayvec::ArrayVec;
        use core::fmt::{self, Display, Formatter};

        type Comments<'input> = ArrayVec<[Comment<'input>; 3]>;
        type Arguments = ArrayVec<[Argument; 10]>;
        type Commands = ArrayVec<[Gcode; 10]>;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source_line: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, source_line: usize) -> Span {
        Span {
            start,
            end,
            source_line,
        }
    }

    pub fn placeholder() -> Span {
        Span {
            start: usize::max_value(),
            end: usize::max_value(),
            source_line: usize::max_value(),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        *self == Span::placeholder()
    }

    pub fn text_from_source<'input>(
        &self,
        src: &'input str,
    ) -> Option<&'input str> {
        src.get(self.start..self.end)
    }

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    src: Option<&'input str>,
    line_number: Option<usize>,
    deleted: bool,
    span: Span,

    #[cfg(not(feature = "std"))]
    commands: Commands,
    #[cfg(feature = "std")]
    commands: Vec<Gcode>,

    #[cfg(not(feature = "std"))]
    comments: Comments<'input>,
    #[cfg(feature = "std")]
    comments: Vec<Comment<'input>>,
}

impl<'input> Block<'input> {
    pub(crate) fn empty() -> Block<'input> {
        Block {
            src: None,
            commands: Default::default(),
            comments: Default::default(),
            deleted: false,
            line_number: None,
            span: Span::placeholder(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
            && self.comments.is_empty()
            && self.line_number.is_none()
    }

    pub fn into_commands(self) -> impl Iterator<Item = Gcode> {
        self.commands.into_iter()
    }

    pub fn commands(&self) -> &[Gcode] {
        &self.commands
    }

    pub fn commands_mut(&mut self) -> &mut [Gcode] {
        &mut self.commands
    }

    pub fn comments(&self) -> &[Comment<'input>] {
        &self.comments
    }

    /// The original source text of the `Block`, if available.
    pub fn src(&self) -> Option<&'input str> {
        self.src
    }

    pub fn with_src(&mut self, src: &'input str) -> &mut Self {
        self.src = Some(src);
        self
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn line_number(&self) -> Option<usize> {
        self.line_number
    }

    pub fn with_line_number(&mut self, number: usize, span: Span) -> &mut Self {
        self.merge_span(span);
        self.line_number = Some(number);
        self
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn delete(&mut self, delete: bool) {
        self.deleted = delete;
    }

    pub fn push_comment(&mut self, comment: Comment<'input>) {
        self.merge_span(comment.span);
        self.comments.push(comment);
    }

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

    #[cfg(not(feature = "std"))]
    arguments: Arguments,
    #[cfg(feature = "std")]
    arguments: Vec<Argument>,
}

impl Gcode {
    pub fn new(mnemonic: Mnemonic, number: f32) -> Gcode {
        debug_assert!(number >= 0.0, "The number should always be positive");
        Gcode {
            mnemonic,
            number: number as f32,
            line_number: None,
            arguments: Default::default(),
            span: Default::default(),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }

    pub fn major_number(&self) -> usize {
        debug_assert!(
            self.number >= 0.0,
            "The number should always be positive"
        );
        self.number.trunc() as usize
    }

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

    pub fn args(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn args_mut(&mut self) -> &mut [Argument] {
        &mut self.arguments
    }

    pub fn with_line_number(&mut self, number: usize) -> &mut Self {
        self.line_number = Some(number);
        self
    }

    pub fn with_minor_nujmber(&mut self, number: usize) -> &mut Self {
        debug_assert!(number < 10);
        self.number = self.number.trunc() + number as f32 / 10.0;
        self
    }

    pub fn with_argument(&mut self, arg: Argument) -> &mut Self {
        self.arguments.push(arg);
        self
    }

    pub fn with_span(&mut self, span: Span) -> &mut Self {
        self.span = span;
        self
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value_for(&self, letter: char) -> Option<f32> {
        let letter = letter.to_ascii_lowercase();

        self.args()
            .into_iter()
            .find(|word| word.letter.to_ascii_lowercase() == letter)
            .map(|word| word.value)
    }
}

/// A command argument.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Argument {
    pub letter: char,
    pub value: f32,
    pub span: Span,
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.value)
    }
}

/// A comment.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Comment<'input> {
    pub body: &'input str,
    pub span: Span,
}

/// The general category a `Gcode` belongs to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Mnemonic {
    General,
    Miscellaneous,
    ProgramNumber,
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
}
