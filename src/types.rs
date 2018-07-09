use number::{Number, Ten};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gcode {
    pub mnemonic: Mnemonic,
    pub number: Number<Ten>,
    pub span: Span,
}

impl Gcode {
    pub fn new(mnemonic: Mnemonic, number: Number<Ten>, span: Span) -> Gcode {
        Gcode { mnemonic, number, span }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mnemonic {
    /// A program number (`O555`).
    ProgramNumber,
    /// A tool change command (`T6`).
    ToolChange,
    /// A machine-specific routine (`M3`).
    MachineRoutine,
    /// A general command (`G01`).
    General,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source_line: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, source_line: usize) -> Span {
        Span { start, end, source_line }
    }
}
