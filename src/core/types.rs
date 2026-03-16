use core::fmt::{self, Display, Formatter};

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

/// A g-code command or line number: major (e.g. the `1` in `G01`) and optional
/// minor (e.g. the `2` in `O0002`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Number {
    /// The main numeric part (e.g. 1 for `G01`, 30 for `M30`).
    pub major: u16,
    /// Optional minor part (e.g. for `O0002` program numbers).
    pub minor: Option<u16>,
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Number { major, minor } = self;
        write!(f, "{}", major)?;
        if let Some(minor) = minor {
            write!(f, ".{}", minor)?;
        }
        Ok(())
    }
}

/// Callbacks invoked by the parser when it encounters invalid or unexpected input.
///
/// The parser does not abort on these conditions; it reports via your
/// implementation and continues. Override the default no-op implementations to
/// collect or log diagnostics. The [`crate::ast`] module provides a
/// diagnostics type that implements this trait and accumulates messages.
#[allow(unused_variables)]
pub trait Diagnostics {
    /// Called when the parser sees text it cannot interpret (e.g. invalid tokens).
    fn emit_unknown_content(&mut self, text: &str, span: Span) {}

    /// Called when the parser expected one of `expected` but found `actual`.
    fn emit_unexpected(&mut self, actual: &str, expected: &[&str], span: Span) {
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
    fn line_number(&mut self, n: Number, span: Span) {}
    /// Comment content (excluding parentheses). Called for each comment on the line.
    fn comment(&mut self, value: &str, span: Span) {}
    /// Optional O program number (e.g. `O0001`). Called at most once per block.
    fn program_number(&mut self, number: Number, span: Span) {}

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
