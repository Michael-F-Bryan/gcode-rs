//! Example: custom visitor that pretty-prints a G-Code program.
//!
//! This implements the core visitor traits (`ProgramVisitor`, `BlockVisitor`,
//! `CommandVisitor`) and is driven by `gcode::core::parse`. As the parser
//! encounters blocks, line numbers, comments, and commands, it calls into
//! this visitor, which formats them into a single string. No AST is built.

use std::fmt::Write;

use gcode::core::{
    BlockVisitor, CommandVisitor, ControlFlow, Diagnostics, HasDiagnostics,
    Number, ProgramVisitor, Span, Value,
};

/// Format a g-code argument value for output (literal number or variable ref).
fn format_value(value: Value<'_>, out: &mut String) {
    match value {
        Value::Literal(n) => {
            write!(out, "{}", n).unwrap();
        },
        Value::Variable(s) => {
            write!(out, "#{}", s).unwrap();
        },
    }
}

/// Top-level visitor: owns the output buffer and delegates each block to a block visitor.
struct PrettyPrinter<'a> {
    output: &'a mut String,
    diagnostics: NoopDiagnostics,
}

struct NoopDiagnostics;

impl Diagnostics for NoopDiagnostics {}

impl HasDiagnostics for PrettyPrinter<'_> {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics {
        &mut self.diagnostics
    }
}

impl ProgramVisitor for PrettyPrinter<'_> {
    fn start_block(&mut self) -> ControlFlow<impl BlockVisitor + '_> {
        ControlFlow::Continue(PrettyPrintBlock {
            output: self.output,
            current_line: String::new(),
            diagnostics: &mut self.diagnostics,
        })
    }
}

/// Block visitor: builds one line of output, then appends it (with newline) in `end_line`.
struct PrettyPrintBlock<'a> {
    output: &'a mut String,
    current_line: String,
    diagnostics: &'a mut NoopDiagnostics,
}

impl PrettyPrintBlock<'_> {
    fn space_if_needed(&mut self) {
        if !self.current_line.is_empty() {
            self.current_line.push(' ');
        }
    }
}

impl HasDiagnostics for PrettyPrintBlock<'_> {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics {
        self.diagnostics
    }
}

impl BlockVisitor for PrettyPrintBlock<'_> {
    fn line_number(&mut self, n: Number, _span: Span) {
        self.space_if_needed();
        write!(self.current_line, "N{}", n).unwrap();
    }

    fn comment(&mut self, value: &str, _span: Span) {
        self.space_if_needed();
        self.current_line.push_str(value);
    }

    fn program_number(&mut self, number: Number, _span: Span) {
        self.space_if_needed();
        write!(self.current_line, "O{}", number).unwrap();
    }

    fn program_delimiter(&mut self, _span: Span) {
        self.space_if_needed();
        self.current_line.push('%');
    }

    fn word_address(&mut self, letter: char, value: Value<'_>, _span: Span) {
        self.space_if_needed();
        self.current_line.push(letter);
        format_value(value, &mut self.current_line);
    }

    fn start_general_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        self.space_if_needed();
        write!(self.current_line, "G{}", number).unwrap();
        ControlFlow::Continue(PrettyPrintCommand {
            line: &mut self.current_line,
            diagnostics: self.diagnostics,
        })
    }

    fn start_miscellaneous_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        self.space_if_needed();
        write!(self.current_line, "M{}", number).unwrap();
        ControlFlow::Continue(PrettyPrintCommand {
            line: &mut self.current_line,
            diagnostics: self.diagnostics,
        })
    }

    fn start_tool_change_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl CommandVisitor + '_> {
        self.space_if_needed();
        write!(self.current_line, "T{}", number).unwrap();
        ControlFlow::Continue(PrettyPrintCommand {
            line: &mut self.current_line,
            diagnostics: self.diagnostics,
        })
    }

    fn end_line(self, _span: Span) {
        self.output.push_str(&self.current_line);
        self.output.push('\n');
    }
}

/// Command visitor: appends each argument (e.g. ` X10.5`, ` Y#1`) to the block's line.
struct PrettyPrintCommand<'a> {
    line: &'a mut String,
    diagnostics: &'a mut NoopDiagnostics,
}

impl HasDiagnostics for PrettyPrintCommand<'_> {
    fn diagnostics(&mut self) -> &mut dyn Diagnostics {
        self.diagnostics
    }
}

impl CommandVisitor for PrettyPrintCommand<'_> {
    fn argument(&mut self, letter: char, value: Value<'_>, _span: Span) {
        self.line.push(' ');
        self.line.push(letter);
        format_value(value, self.line);
    }

    fn end_command(self, _span: Span) {}
}

fn main() {
    let src = r"
N10 G21 G90 (metric, absolute)
N20 G00 X50.0 Y-10.0
N30 M03 S12000
N40 G01 X1.5 Y-0.25 F100
";
    let mut output = String::new();
    let mut visitor = PrettyPrinter {
        output: &mut output,
        diagnostics: NoopDiagnostics,
    };
    gcode::core::parse(src, &mut visitor);
    println!("Pretty-printed program:\n{}", output);
}
