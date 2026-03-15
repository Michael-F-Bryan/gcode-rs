//! A zero-allocation push-based parser for g-code.
//!
//! # How the API works
//!
//! Parsing is **push-based**: the parser drives the visitor. You implement
//! visitor traits and pass a visitor into [`parse`]; the parser then calls your
//! methods as it encounters each item in the source.
//!
//! **Nested visitors:** The API is structured in layers. A [`ProgramVisitor`]
//! receives [`start_line`](ProgramVisitor::start_line) and returns a
//! [`ControlFlow`] containing a [`LineVisitor`] for that line. The line
//! visitor receives line numbers, comments, and—when a G/M/O/T command
//! starts—[`start_g_code`](LineVisitor::start_g_code), which returns a
//! [`ControlFlow`] containing a [`GCodeVisitor`] for that command. Each
//! visitor can be a new value that borrows from the parent, so the whole chain
//! can be zero-allocation.
//!
//! **Pausing and resuming:** If a visitor returns [`ControlFlow::Break`], the
//! parser stops (e.g. because a buffer is full). To resume, call
//! [`Parser::parse`] again with the same parser and visitor; parsing
//! continues from where it left off. [`ControlFlow::Skip`] skips the current
//! item without stopping. [`ControlFlow::Continue(visitor)`] supplies the
//! next-level visitor and continues.
//!
//! **Errors:** Recoverable errors are reported via callback methods on the
//! visitor for the level where they occur (e.g. unexpected line number on
//! [`LineVisitor`], argument overflow on [`GCodeVisitor`]). The parser does
//! not abort; it calls the callback and continues.

#![allow(missing_docs)]

use core::fmt::{self, Display, Formatter};

fn f32_to_number(v: f32) -> Number {
    let major = v.trunc();
    let minor = (v.fract() * 10.0).round();
    Number {
        major: major as u16,
        minor: if minor == 0.0 {
            None
        } else {
            Some(minor as u16)
        },
    }
}

#[derive(Debug)]
enum LineItem<'a> {
    Word(char, f32, usize, usize),
    Comment(&'a str, usize, usize),
    Unknown(&'a str, usize, usize),
    LetterOnly(char, usize, usize),
    NumberOnly(&'a str, usize, usize),
}

fn parse_line_items(line: &str, line_start: usize) -> Vec<LineItem<'_>> {
    let mut items = Vec::new();
    let mut i = 0;
    let bytes = line.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        if bytes[i] == b';' {
            i += 1;
            let comment_start = i;
            while i < bytes.len() {
                i += 1;
            }
            items.push(LineItem::Comment(
                &line[comment_start..i],
                line_start + start,
                line_start + i,
            ));
            continue;
        }
        if bytes[i] == b'(' {
            let comment_start = start;
            i += 1;
            while i < bytes.len() && bytes[i] != b')' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            let value = &line[comment_start..i];
            items.push(LineItem::Comment(
                value,
                line_start + comment_start,
                line_start + i,
            ));
            continue;
        }
        if (bytes[i] >= b'A' && bytes[i] <= b'Z') || (bytes[i] >= b'a' && bytes[i] <= b'z') {
            let letter = line[i..].chars().next().unwrap();
            i += letter.len_utf8();
            let num_start = i;
            if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
                i += 1;
            }
            let mut has_digit = false;
            while i < bytes.len() && (bytes[i] >= b'0' && bytes[i] <= b'9') {
                has_digit = true;
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                while i < bytes.len() && (bytes[i] >= b'0' && bytes[i] <= b'9') {
                    has_digit = true;
                    i += 1;
                }
            }
            if has_digit {
                let num_str = &line[num_start..i];
                if let Ok(v) = num_str.parse::<f32>() {
                    items.push(LineItem::Word(
                        letter,
                        v,
                        line_start + start,
                        line_start + i,
                    ));
                } else {
                    items.push(LineItem::LetterOnly(
                        letter,
                        line_start + start,
                        line_start + i,
                    ));
                }
            } else {
                items.push(LineItem::LetterOnly(
                    letter,
                    line_start + start,
                    line_start + num_start,
                ));
            }
            continue;
        }
        if bytes[i] >= b'0' && bytes[i] <= b'9' || bytes[i] == b'.' || bytes[i] == b'-' || bytes[i] == b'+' {
            let num_start = i;
            if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
                i += 1;
            }
            while i < bytes.len() && (bytes[i] >= b'0' && bytes[i] <= b'9' || bytes[i] == b'.') {
                i += 1;
            }
            items.push(LineItem::NumberOnly(
                &line[num_start..i],
                line_start + num_start,
                line_start + i,
            ));
            continue;
        }
        let unknown_start = i;
        i += 1;
        while i < bytes.len()
            && bytes[i] != b' '
            && bytes[i] != b'\t'
            && bytes[i] != b';'
            && bytes[i] != b'('
            && (bytes[i] < b'A' || bytes[i] > b'Z')
            && (bytes[i] < b'a' || bytes[i] > b'z')
            && (bytes[i] < b'0' || bytes[i] > b'9')
        {
            i += 1;
        }
        items.push(LineItem::Unknown(
            &line[unknown_start..i],
            line_start + unknown_start,
            line_start + i,
        ));
    }
    items
}

struct CommandVisitorProxy<'a>(Option<Box<dyn CommandVisitor + 'a>>);

impl CommandVisitor for CommandVisitorProxy<'_> {
    fn argument(&mut self, letter: char, value: f32, span: Span) {
        if let Some(b) = &mut self.0 {
            b.argument(letter, value, span);
        }
    }
    fn argument_buffer_overflow_error(&mut self, letter: char, value: f32, span: Span) {
        if let Some(b) = &mut self.0 {
            b.argument_buffer_overflow_error(letter, value, span);
        }
    }
}

struct BoxingLineVisitor<'a, LV: LineVisitor + ?Sized> {
    inner: &'a mut LV,
}

#[allow(refining_impl_trait)]
impl<LV: LineVisitor> LineVisitor for BoxingLineVisitor<'_, LV> {
    fn line_number(&mut self, n: f32, span: Span) {
        self.inner.line_number(n, span);
    }
    fn comment(&mut self, value: &str, span: Span) {
        self.inner.comment(value, span);
    }
    fn program_number(&mut self, number: Number, span: Span) {
        self.inner.program_number(number, span);
    }
    fn start_general_code(
        &mut self,
        number: Number,
        span: Span,
    ) -> ControlFlow<CommandVisitorProxy<'_>> {
        match self.inner.start_general_code(number, span) {
            ControlFlow::Continue(c) => {
                ControlFlow::Continue(CommandVisitorProxy(Some(Box::new(c))))
            }
            ControlFlow::Break => ControlFlow::Break,
        }
    }
    fn start_miscellaneous_code(
        &mut self,
        number: Number,
        span: Span,
    ) -> ControlFlow<CommandVisitorProxy<'_>> {
        match self.inner.start_miscellaneous_code(number, span) {
            ControlFlow::Continue(c) => {
                ControlFlow::Continue(CommandVisitorProxy(Some(Box::new(c))))
            }
            ControlFlow::Break => ControlFlow::Break,
        }
    }
    fn start_tool_change_code(
        &mut self,
        number: Number,
        span: Span,
    ) -> ControlFlow<CommandVisitorProxy<'_>> {
        match self.inner.start_tool_change_code(number, span) {
            ControlFlow::Continue(c) => {
                ControlFlow::Continue(CommandVisitorProxy(Some(Box::new(c))))
            }
            ControlFlow::Break => ControlFlow::Break,
        }
    }
    fn unknown_content_error(&mut self, text: &str, span: Span) {
        self.inner.unknown_content_error(text, span);
    }
    fn unexpected_line_number_error(&mut self, n: f32, span: Span) {
        self.inner.unexpected_line_number_error(n, span);
    }
    fn letter_without_number_error(&mut self, value: &str, span: Span) {
        self.inner.letter_without_number_error(value, span);
    }
    fn number_without_letter_error(&mut self, value: &str, span: Span) {
        self.inner.number_without_letter_error(value, span);
    }
}

#[allow(unused_assignments)] // cmd_visitor = None drops previous proxy before reassigning
fn feed_line(
    line: &str,
    line_start: usize,
    line_index: usize,
    line_visitor: &mut impl LineVisitor,
) {
    let mut boxing = BoxingLineVisitor {
        inner: line_visitor,
    };
    let items = parse_line_items(line, line_start);
    let mut cmd_visitor: Option<CommandVisitorProxy<'_>> = None;
    let mut seen_command_on_line = false;
    for item in items {
        match item {
            LineItem::Word(letter, value, start, end) => {
                let span = Span::new(start, end, line_index);
                let num = f32_to_number(value);
                match letter {
                    'G' | 'g' => {
                        cmd_visitor = None;
                        seen_command_on_line = true;
                        cmd_visitor = Some(match boxing.start_general_code(num, span) {
                            ControlFlow::Continue(c) => c,
                            ControlFlow::Break => return,
                        });
                    }
                    'M' | 'm' => {
                        cmd_visitor = None;
                        seen_command_on_line = true;
                        cmd_visitor = Some(match boxing.start_miscellaneous_code(num, span) {
                            ControlFlow::Continue(c) => c,
                            ControlFlow::Break => return,
                        });
                    }
                    'O' | 'o' => {
                        cmd_visitor = None;
                        seen_command_on_line = true;
                        boxing.program_number(num, span);
                    }
                    'T' | 't' => {
                        cmd_visitor = None;
                        seen_command_on_line = true;
                        cmd_visitor = Some(match boxing.start_tool_change_code(num, span) {
                            ControlFlow::Continue(c) => c,
                            ControlFlow::Break => return,
                        });
                    }
                    'N' | 'n' => {
                        cmd_visitor = None;
                        if seen_command_on_line {
                            boxing.unexpected_line_number_error(value, span);
                        } else {
                            boxing.line_number(value, span);
                        }
                    }
                    _ => {
                        if let Some(ref mut cv) = cmd_visitor {
                            cv.argument(letter, value, span);
                        }
                    }
                }
            }
            LineItem::Comment(value, start, end) => {
                cmd_visitor = None;
                boxing.comment(value, Span::new(start, end, line_index));
            }
            LineItem::Unknown(text, start, end) => {
                cmd_visitor = None;
                boxing.unknown_content_error(
                    text,
                    Span::new(start, end, line_index),
                );
            }
            LineItem::LetterOnly(_value, start, end) => {
                cmd_visitor = None;
                let span = Span::new(start, end, line_index);
                let s = &line[(start - line_start)..(end - line_start)];
                boxing.letter_without_number_error(s, span);
            }
            LineItem::NumberOnly(value, start, end) => {
                cmd_visitor = None;
                boxing.number_without_letter_error(
                    value,
                    Span::new(start, end, line_index),
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Number {
    pub major: u16,
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

pub fn parse(src: &str, visitor: &mut impl ProgramVisitor) {
    let mut parser = Parser::new(src);
    parser.parse(visitor);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlFlow<T> {
    Continue(T),
    Break,
}

pub trait ProgramVisitor {
    fn start_line(&mut self, span: Span) -> ControlFlow<impl LineVisitor + '_>;
}

pub trait LineVisitor {
    fn line_number(&mut self, _n: f32, _span: Span) {}
    fn comment(&mut self, _value: &str, _span: Span) {}
    fn program_number(&mut self, _number: Number, _span: Span) {}

    fn start_general_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_miscellaneous_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }
    fn start_tool_change_code(
        &mut self,
        _number: Number,
        _span: Span,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        ControlFlow::Continue(Noop)
    }

    fn unknown_content_error(&mut self, _text: &str, _span: Span) {}
    fn unexpected_line_number_error(&mut self, _n: f32, _span: Span) {}
    fn letter_without_number_error(&mut self, _value: &str, _span: Span) {}
    fn number_without_letter_error(&mut self, _value: &str, _span: Span) {}
}

pub trait CommandVisitor {
    fn argument(&mut self, _letter: char, _value: f32, _span: Span) {}

    fn argument_buffer_overflow_error(
        &mut self,
        _letter: char,
        _value: f32,
        _span: Span,
    ) {
    }
}

struct Noop;

impl CommandVisitor for Noop {}

impl LineVisitor for Noop {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parser<'src> {
    src: &'src str,
    current_index: usize,
}

impl<'src> Parser<'src> {
    pub const fn new(src: &'src str) -> Self {
        Self {
            src,
            current_index: 0,
        }
    }

    pub fn finished(&self) -> bool {
        self.current_index >= self.src.len()
    }

    pub fn parse(&mut self, visitor: &mut impl ProgramVisitor) {
        let rest = self.src.get(self.current_index..).unwrap_or("");
        let mut line_start = self.current_index;
        let mut line_index: usize = 0;
        for line in rest.lines() {
            let line_end = line_start + line.len();
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let span = Span::new(line_start, line_end, line_index);
                let mut line_visitor = match visitor.start_line(span) {
                    ControlFlow::Continue(lv) => lv,
                    ControlFlow::Break => return,
                };
                feed_line(trimmed, line_start, line_index, &mut line_visitor);
            }
            line_index += 1;
            line_start = line_end + 1;
            if line_start > self.src.len() {
                line_start = self.src.len();
            }
        }
        self.current_index = line_start;
    }
}

/// A half-open range which indicates the location of something in a body of
/// text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
#[repr(C)]
pub struct Span {
    /// The byte index corresponding to the item's start.
    pub start: usize,
    /// The index one byte past the item's end.
    pub end: usize,
    /// The (zero-based) line number.
    pub line: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize, line: usize) -> Self {
        assert!(start <= end);
        Self { start, end, line }
    }
}

#[cfg(test)]
#[allow(refining_impl_trait)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        LineStarted(Span),
        LineNumber(f32, Span),
        Comment(String, Span),
        GeneralCode(Number, Span),
        MiscCode(Number, Span),
        ToolChangeCode(Number, Span),
        ProgramNumber(Number, Span),
        Argument(char, f32, Span),
        UnknownContentError(String, Span),
        UnexpectedLineNumberError(f32, Span),
        LetterWithoutNumberError(String, Span),
        NumberWithoutLetterError(String, Span),
    }

    struct RecordingProgramVisitor {
        events: Vec<Event>,
    }

    impl RecordingProgramVisitor {
        fn new() -> Self {
            Self {
                events: Vec::new(),
            }
        }
    }

    pub(super) struct RecordingLineVisitor<'a> {
        events: &'a mut Vec<Event>,
        _line_span: Span,
    }

    pub(super) struct RecordingCommandVisitor<'a> {
        events: &'a mut Vec<Event>,
    }

    impl ProgramVisitor for RecordingProgramVisitor {
        fn start_line(&mut self, span: Span) -> ControlFlow<RecordingLineVisitor<'_>> {
            self.events.push(Event::LineStarted(span));
            ControlFlow::Continue(RecordingLineVisitor {
                events: &mut self.events,
                _line_span: span,
            })
        }
    }

    impl LineVisitor for RecordingLineVisitor<'_> {
        fn line_number(&mut self, n: f32, span: Span) {
            self.events.push(Event::LineNumber(n, span));
        }
        fn comment(&mut self, value: &str, span: Span) {
            self.events
                .push(Event::Comment(value.to_string(), span));
        }
        fn program_number(&mut self, number: Number, span: Span) {
            self.events.push(Event::ProgramNumber(number, span));
        }
        fn start_general_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<RecordingCommandVisitor<'_>> {
            self.events.push(Event::GeneralCode(number, span));
            ControlFlow::Continue(RecordingCommandVisitor {
                events: self.events,
            })
        }
        fn start_miscellaneous_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<RecordingCommandVisitor<'_>> {
            self.events.push(Event::MiscCode(number, span));
            ControlFlow::Continue(RecordingCommandVisitor {
                events: self.events,
            })
        }
        fn start_tool_change_code(
            &mut self,
            number: Number,
            span: Span,
        ) -> ControlFlow<RecordingCommandVisitor<'_>> {
            self.events.push(Event::ToolChangeCode(number, span));
            ControlFlow::Continue(RecordingCommandVisitor {
                events: self.events,
            })
        }
        fn unknown_content_error(&mut self, text: &str, span: Span) {
            self.events
                .push(Event::UnknownContentError(text.to_string(), span));
        }
        fn unexpected_line_number_error(&mut self, n: f32, span: Span) {
            self.events.push(Event::UnexpectedLineNumberError(n, span));
        }
        fn letter_without_number_error(&mut self, value: &str, span: Span) {
            self.events
                .push(Event::LetterWithoutNumberError(value.to_string(), span));
        }
        fn number_without_letter_error(&mut self, value: &str, span: Span) {
            self.events
                .push(Event::NumberWithoutLetterError(value.to_string(), span));
        }
    }

    impl CommandVisitor for RecordingCommandVisitor<'_> {
        fn argument(&mut self, letter: char, value: f32, span: Span) {
            self.events.push(Event::Argument(letter, value, span));
        }
    }

    fn parse_and_record(src: &str) -> Vec<Event> {
        let mut visitor = RecordingProgramVisitor::new();
        parse(src, &mut visitor);
        visitor.events
    }

    #[test]
    fn empty_input_produces_no_events() {
        let events = parse_and_record("");
        assert!(events.is_empty(), "expected no events, got {:?}", events);
    }

    #[test]
    fn single_g_code_no_args() {
        let events = parse_and_record("G90");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::GeneralCode(n, _) => {
                assert_eq!(n.major, 90);
                assert_eq!(n.minor, None);
            }
            _ => panic!("expected GeneralCode(90), got {:?}", events[1]),
        }
    }

    #[test]
    fn g_code_with_arguments() {
        let events = parse_and_record("G01 X10 Y-20");
        assert_eq!(events.len(), 4); // LineStarted, GeneralCode(1), Argument(X,10), Argument(Y,-20)
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::GeneralCode(n, _) => {
                assert_eq!(n.major, 1);
                assert_eq!(n.minor, None);
            }
            _ => panic!("expected GeneralCode(1), got {:?}", events[1]),
        }
        match &events[2] {
            Event::Argument('X', v, _) => assert!((*v - 10.0).abs() < 1e-6),
            _ => panic!("expected Argument(X, 10), got {:?}", events[2]),
        }
        match &events[3] {
            Event::Argument('Y', v, _) => assert!((*v - (-20.0)).abs() < 1e-6),
            _ => panic!("expected Argument(Y, -20), got {:?}", events[3]),
        }
    }

    #[test]
    fn comment_semicolon() {
        let events = parse_and_record("; hello world");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::Comment(s, _) => assert_eq!(s, " hello world"),
            _ => panic!("expected Comment, got {:?}", events[1]),
        }
    }

    #[test]
    fn comment_parens() {
        let events = parse_and_record("(Linear / Feed - Absolute)");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::Comment(s, _) => assert_eq!(s, "(Linear / Feed - Absolute)"),
            _ => panic!("expected Comment, got {:?}", events[1]),
        }
    }

    #[test]
    fn line_number_then_g_code() {
        let events = parse_and_record("N42 G90");
        assert_eq!(events.len(), 3); // LineStarted, LineNumber(42), GeneralCode(90)
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::LineNumber(n, _) => assert_eq!(*n, 42.0),
            _ => panic!("expected LineNumber(42), got {:?}", events[1]),
        }
        match &events[2] {
            Event::GeneralCode(n, _) => {
                assert_eq!(n.major, 90);
                assert_eq!(n.minor, None);
            }
            _ => panic!("expected GeneralCode(90), got {:?}", events[2]),
        }
    }

    #[test]
    fn program_number_o_code() {
        let events = parse_and_record("O1000");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::ProgramNumber(n, _) => {
                assert_eq!(n.major, 1000);
                assert_eq!(n.minor, None);
            }
            _ => panic!("expected ProgramNumber(1000), got {:?}", events[1]),
        }
    }

    #[test]
    fn miscellaneous_code_with_arg() {
        let events = parse_and_record("M3 S1000");
        assert_eq!(events.len(), 3); // LineStarted, MiscCode(3), Argument(S, 1000)
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::MiscCode(n, _) => assert_eq!(n.major, 3),
            _ => panic!("expected MiscCode(3), got {:?}", events[1]),
        }
        match &events[2] {
            Event::Argument('S', v, _) => assert!((*v - 1000.0).abs() < 1e-6),
            _ => panic!("expected Argument(S, 1000), got {:?}", events[2]),
        }
    }

    #[test]
    fn tool_change_code() {
        let events = parse_and_record("T2");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::LineStarted(_)));
        match &events[1] {
            Event::ToolChangeCode(n, _) => assert_eq!(n.major, 2),
            _ => panic!("expected ToolChangeCode(2), got {:?}", events[1]),
        }
    }

    #[test]
    fn multiple_codes_same_line() {
        let events = parse_and_record("G0 G90 G40 G21");
        assert_eq!(events.len(), 5); // LineStarted + 4 GeneralCode
        assert!(matches!(events[0], Event::LineStarted(_)));
        for (i, expected_major) in [0u16, 90, 40, 21].iter().enumerate() {
            match &events[1 + i] {
                Event::GeneralCode(n, _) => assert_eq!(n.major, *expected_major),
                _ => panic!("expected GeneralCode({}), got {:?}", expected_major, events[1 + i]),
            }
        }
    }

    #[test]
    fn two_lines() {
        let events = parse_and_record("G90\nG01 X1");
        assert_eq!(events.len(), 5); // LineStarted, GeneralCode(90), LineStarted, GeneralCode(1), Argument(X,1)
        assert!(matches!(events[0], Event::LineStarted(_)));
        assert!(matches!(events[1], Event::GeneralCode(Number { major: 90, .. }, _)));
        assert!(matches!(events[2], Event::LineStarted(_)));
        assert!(matches!(events[3], Event::GeneralCode(Number { major: 1, .. }, _)));
        match &events[4] {
            Event::Argument('X', v, _) => assert!((*v - 1.0).abs() < 1e-6),
            _ => panic!("expected Argument(X, 1), got {:?}", events[4]),
        }
    }

    #[test]
    fn decimal_argument() {
        let events = parse_and_record("G01 X1.5 Y-0.25");
        assert!(events.len() >= 4);
        let args: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::Argument(l, v, _) => Some((*l, *v)),
                _ => None,
            })
            .collect();
        assert_eq!(args.len(), 2);
        assert!((args[0].1 - 1.5).abs() < 1e-6);
        assert!((args[1].1 - (-0.25)).abs() < 1e-6);
    }

    #[test]
    fn minor_subcode_g91_1() {
        let events = parse_and_record("G91.1");
        assert_eq!(events.len(), 2);
        match &events[1] {
            Event::GeneralCode(n, _) => {
                assert_eq!(n.major, 91);
                assert_eq!(n.minor, Some(1));
            }
            _ => panic!("expected GeneralCode(91.1), got {:?}", events[1]),
        }
    }

    #[test]
    fn whitespace_only_input() {
        let events = parse_and_record("   \n\t  ");
        assert!(events.is_empty());
    }

    #[test]
    fn no_space_between_words() {
        let events = parse_and_record("G00G21G17G90");
        assert_eq!(events.len(), 5); // LineStarted + 4 GeneralCode
        for (i, expected) in [0u16, 21, 17, 90].iter().enumerate() {
            match &events[1 + i] {
                Event::GeneralCode(n, _) => assert_eq!(n.major, *expected),
                _ => panic!("expected GeneralCode({}), got {:?}", expected, events[1 + i]),
            }
        }
    }

    #[test]
    fn unknown_content_error() {
        let events = parse_and_record("G90 $$%# X10");
        let errors: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::UnknownContentError(t, _) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        assert!(!errors.is_empty(), "expected at least one UnknownContentError, got {:?}", events);
    }

    #[test]
    fn unexpected_line_number_error() {
        let events = parse_and_record("G90 N42");
        let errors: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::UnexpectedLineNumberError(n, _) => Some(*n),
                _ => None,
            })
            .collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], 42.0);
    }

    #[test]
    fn letter_without_number_error() {
        let events = parse_and_record("G");
        let errors: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::LetterWithoutNumberError(t, _) => Some(t.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "G");
    }

    #[test]
    fn number_without_letter_error() {
        let events = parse_and_record("42");
        let errors: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::NumberWithoutLetterError(t, _) => Some(t.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "42");
    }

    #[test]
    fn code_then_comment_same_line() {
        let events = parse_and_record("G90 ; absolute mode");
        assert_eq!(events.len(), 3); // LineStarted, GeneralCode(90), Comment
        assert!(matches!(events[0], Event::LineStarted(_)));
        assert!(matches!(events[1], Event::GeneralCode(Number { major: 90, .. }, _)));
        match &events[2] {
            Event::Comment(s, _) => assert!(s.contains("absolute")),
            _ => panic!("expected Comment, got {:?}", events[2]),
        }
    }

    #[test]
    fn regression_fixed_snippet_event_sequence() {
        let events = parse_and_record("N10 G0 X1 Y2");
        let expected: Vec<&str> = events
            .iter()
            .map(|e| match e {
                Event::LineStarted(_) => "LineStarted",
                Event::LineNumber(..) => "LineNumber",
                Event::GeneralCode(..) => "GeneralCode",
                Event::Argument(..) => "Argument",
                _ => "Other",
            })
            .collect();
        assert_eq!(
            expected,
            ["LineStarted", "LineNumber", "GeneralCode", "Argument", "Argument"],
            "event sequence should be stable for N10 G0 X1 Y2"
        );
    }
}
