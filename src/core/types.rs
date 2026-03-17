use core::{
    fmt::{self, Debug, Display, Formatter},
    num::NonZeroU32,
};

/// Lexer token kind. Used by the parser and in [`Diagnostics::emit_unexpected`] to
/// report what was expected.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TokenType {
    /// Single alphabetic character that is not G, M, or T (e.g. N, O, X).
    Letter,
    /// G code letter (case-insensitive in source).
    G,
    /// M code letter (case-insensitive in source).
    M,
    /// T code letter (case-insensitive in source).
    T,
    /// Program delimiter `%` (RS-274 / ISO 6983).
    Percent,
    /// Numeric literal.
    Number,
    /// Comment (semicolon or parentheses).
    Comment,
    /// A `/` is used to indicate a deleted block.
    Slash,
    /// Minus sign.
    MinusSign,
    /// Plus sign.
    PlusSign,
    /// Newline.
    Newline,
    /// Unrecognised token.
    Unknown,
    /// Virtual: expected end of input (no more tokens).
    Eof,
}

impl TokenType {
    /// Get the human-friendly name for this token type.
    pub const fn as_str(self) -> &'static str {
        match self {
            TokenType::Letter => "letter",
            TokenType::G => "G",
            TokenType::M => "M",
            TokenType::T => "T",
            TokenType::Percent => "%",
            TokenType::Number => "number",
            TokenType::Comment => "comment",
            TokenType::Slash => "slash",
            TokenType::MinusSign => "minus sign",
            TokenType::PlusSign => "plus sign",
            TokenType::Newline => "newline",
            TokenType::Unknown => "unknown",
            TokenType::Eof => "eof",
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Return type for visitor methods that may either continue with a child visitor
/// or pause parsing. See the [module-level docs](crate::core) for the control-flow model.
pub type ControlFlow<T> = core::ops::ControlFlow<(), T>;

/// A half-open range indicating where an element appears in the source text.
///
/// Use [`start`](Span::start) and [`end`](Span::end) (or `start + length`) for
/// byte offsets; [`line`](Span::line) is the zero-based line number. All spans
/// refer into the same `&str` passed to [`parse`](crate::core::parse) or
/// [`resume`](crate::core::resume).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Span {
    /// The byte index corresponding to the item's start.
    pub start: usize,
    /// The number of bytes in the span.
    pub length: usize,
    /// The (zero-based) line number.
    pub line: usize,
}

impl Span {
    /// Constructs a span from start index, length in bytes, and line number.
    pub const fn new(start: usize, length: usize, line: usize) -> Self {
        Self {
            start,
            length,
            line,
        }
    }

    /// The index one byte past the item's end.
    #[must_use]
    pub const fn end(self) -> usize {
        self.start + self.length
    }
}

/// A g-code command number (G, M, T): fixed-point decimal premultiplied by 10.
/// E.g. `G91.1` → `Number::new_with_minor(91, 1)`.
/// Line numbers (N) and program numbers (O) use `u32`; see
/// [`BlockVisitor::line_number`](BlockVisitor::line_number) and
/// [`BlockVisitor::program_number`](BlockVisitor::program_number).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Number(u32);

impl Number {
    const SCALAR: u32 = 10;

    /// Create a new number with no decimals.
    pub const fn new(major: u32) -> Self {
        Self(major * Self::SCALAR)
    }

    /// Create a new number with a minor (decimal) component (e.g. `G91.1`).
    pub const fn new_with_minor(major: u32, minor: u32) -> Self {
        assert!(minor < Self::SCALAR, "Overflow");
        Self(major * Self::SCALAR + minor)
    }

    /// The major component (e.g. the `91` in `G91.1`).
    pub const fn major(self) -> u32 {
        self.0 / Self::SCALAR
    }

    /// The minor (decimal) component (e.g. the `1` in `G91.1`).
    pub const fn minor(self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.0 % Self::SCALAR)
    }
}

impl core::str::FromStr for Number {
    type Err = ParseNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major_str, minor_str) = match s.split_once('.') {
            None => (s, None),
            Some((maj, min)) => (maj, Some(min)),
        };
        let major = major_str.parse::<u32>()?;

        let minor = match minor_str {
            Some(s) => s.parse::<u32>()?,
            None => 0,
        };
        if minor >= Self::SCALAR {
            return Err(ParseNumberError::Overflow);
        }

        let value = major * 10 + minor;
        Ok(Number(value))
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let major = self.major();
        write!(f, "{major}")?;

        if let Some(minor) = self.minor() {
            write!(f, ".{minor}")?;
        }
        Ok(())
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNumberError {
    ParseInt(core::num::ParseIntError),
    Overflow,
}

impl From<core::num::ParseIntError> for ParseNumberError {
    fn from(error: core::num::ParseIntError) -> Self {
        ParseNumberError::ParseInt(error)
    }
}

impl Display for ParseNumberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseNumberError::ParseInt(e) => write!(f, "{}", e),
            ParseNumberError::Overflow => write!(f, "Overflow"),
        }
    }
}

/// Callbacks invoked by the parser when it encounters invalid or unexpected input.
///
/// The parser does not abort on these conditions; it reports via your
/// implementation and continues. Override the default no-op implementations to
/// collect or log diagnostics. The [`crate`] module provides a
/// diagnostics type that implements this trait and accumulates messages.
#[allow(unused_variables)]
pub trait Diagnostics {
    /// Called when the parser sees text it cannot interpret (e.g. invalid tokens).
    fn emit_unknown_content(&mut self, text: &str, span: Span) {}

    /// Called when the parser expected one of `expected` token types but found `actual`.
    fn emit_unexpected(
        &mut self,
        actual: &str,
        expected: &[TokenType],
        span: Span,
    ) {
    }

    /// Called when parsing a G/M/T number fails (e.g. overflow, invalid format).
    fn emit_parse_number_error(
        &mut self,
        value: &str,
        error: ParseNumberError,
        span: Span,
    ) {
    }

    /// Called when parsing an N or O number fails (e.g. invalid integer, overflow).
    fn emit_parse_int_error(
        &mut self,
        value: &str,
        _error: core::num::ParseIntError,
        span: Span,
    ) {
    }
}

/// Allows the parser to obtain a [`Diagnostics`] implementation from a visitor.
///
/// All visitor traits (`ProgramVisitor`, [`BlockVisitor`], [`CommandVisitor`])
/// require this so the parser can report recoverable errors. Implement by
/// returning a mutable reference to your diagnostics; the blanket impl for
/// `D: Diagnostics` lets you use any diagnostics type as its own visitor when
/// you only need to record errors.
pub trait HasDiagnostics {
    /// Returns a mutable reference to the diagnostics sink used for this parse.
    fn diagnostics(&mut self) -> &mut dyn Diagnostics;
}

impl<D: Diagnostics> HasDiagnostics for D {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics {
        &mut *self
    }
}

/// Top-level visitor: one implementation per parse, created by the caller and
/// passed to [`parse`](crate::core::parse) or [`resume`](crate::core::resume).
///
/// The parser calls [`start_block`](ProgramVisitor::start_block) for each block
/// (line) in the source. Return [`ControlFlow::Continue`] with a [`BlockVisitor`]
/// to process that line, or [`ControlFlow::Break`] to pause (e.g. buffer full);
/// the returned [`ParserState`](crate::core::ParserState) can be passed to
/// [`resume`](crate::core::resume) to continue later.
pub trait ProgramVisitor: HasDiagnostics {
    /// Called at the start of each block (line). Return a [`BlockVisitor`] to
    /// handle this block, or break to pause parsing.
    fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_>;
}

/// Visitor for a single block (line) of g-code.
///
/// The parser creates a block visitor from [`ProgramVisitor::start_block`] and
/// then calls the methods below in order. Terminals (line number, comment,
/// program number) are reported with a single call; each G/M/O/T command on the
/// line is entered via one of the `start_*_code` methods, which return a
/// [`CommandVisitor`] for that command. When the line ends, the parser calls
/// [`end_line`](BlockVisitor::end_line), consuming this visitor.
///
/// # Call order
///
/// - Optional: [`line_number`](BlockVisitor::line_number), [`comment`](BlockVisitor::comment),
///   [`program_number`](BlockVisitor::program_number) (each at most once per block, in
///   any order depending on source).
/// - Zero or more commands: [`start_general_code`](BlockVisitor::start_general_code),
///   [`start_miscellaneous_code`](BlockVisitor::start_miscellaneous_code),
///   [`start_tool_change_code`](BlockVisitor::start_tool_change_code), each returning a
///   [`CommandVisitor`] that receives arguments then [`CommandVisitor::end_command`].
/// - Exactly once: [`end_line`](BlockVisitor::end_line).
#[allow(unused_variables)]
pub trait BlockVisitor: HasDiagnostics + Sized {
    /// Optional N line number (e.g. `N100`). Called at most once per block.
    fn line_number(&mut self, n: u32, span: Span) {}
    /// Comment content (excluding parentheses). Called for each comment on the line.
    fn comment(&mut self, value: &str, span: Span) {}
    /// Optional O program number (e.g. `O0001`). Called at most once per block.
    fn program_number(&mut self, number: u32, span: Span) {}
    /// Program delimiter `%` (RS-274 / ISO 6983). Called once per `%` token.
    fn program_delimiter(&mut self, _span: Span) {}
    /// Modal bare word address (e.g. `X5.0`, `S12000` at block level without a G/M/T prefix).
    fn word_address(&mut self, _letter: char, _value: Value<'_>, _span: Span) {}

    /// Start of a G (general) command. Return a [`CommandVisitor`] to handle
    /// this command, or use the default to ignore it.
    fn start_general_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    /// Start of an M (miscellaneous) command. Return a [`CommandVisitor`] to handle
    /// this command, or use the default to ignore it.
    fn start_miscellaneous_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    /// Start of a T (tool change) command. Return a [`CommandVisitor`] to handle
    /// this command, or use the default to ignore it.
    fn start_tool_change_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }

    /// Called once at the end of the block. The parser consumes this visitor
    /// after this call; use it to finalize per-block state.
    fn end_line(self, span: Span) {}
}

/// Visitor for a single command (one G, M, O, or T code and its arguments).
///
/// Created by a [`BlockVisitor`]'s `start_*_code` method. The parser calls
/// [`argument`](CommandVisitor::argument) zero or more times (once per word,
/// e.g. `X10` or `Y-2.5`), then [`end_command`](CommandVisitor::end_command),
/// consuming this visitor.
#[allow(unused_variables)]
pub trait CommandVisitor: HasDiagnostics + Sized {
    /// One argument: address letter (e.g. `'X'`) and its [`Value`] with span.
    fn argument(&mut self, letter: char, value: Value<'_>, span: Span) {}

    /// Called once at the end of the command. The parser consumes this visitor
    /// after this call.
    fn end_command(self, span: Span) {}
}

/// The value of a g-code argument: a numeric literal or a variable reference.
///
/// Borrows from the source string for [`Variable`](Value::Variable); use
/// before the parse ends or copy the value if you need it later.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value<'src> {
    /// A literal number (e.g. `10`, `-2.5`, `0.1e-2`).
    Literal(f32),
    /// A variable reference (e.g. `#1`, `#<expr>`); the string is the raw text
    /// after the `#`, not evaluated.
    Variable(&'src str),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Literal(n) => write!(f, "{}", n),
            Value::Variable(s) => write!(f, "#{}", s),
        }
    }
}

/// A no-op visitor that ignores all callbacks.
///
/// Use when you only need to drive the parser (e.g. to validate syntax or
/// consume input) without building any structure or collecting diagnostics.
///
/// # Example
///
/// ```
/// use gcode::core::{parse, Noop};
///
/// let src = "G90 G01 X5 Y10";
/// parse(src, &mut Noop);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Noop;

impl ProgramVisitor for Noop {
    fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
}

impl CommandVisitor for Noop {}

impl BlockVisitor for Noop {}

impl Diagnostics for Noop {}
