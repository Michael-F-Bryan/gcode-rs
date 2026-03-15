//! Integration tests for the core push-based parser using fixtures from tests/data/.

#![allow(refining_impl_trait)]

use gcode::core::{
    CommandVisitor, ControlFlow, LineVisitor, Number, ProgramVisitor, Span,
    parse,
};

struct NopCommandVisitor;
impl CommandVisitor for NopCommandVisitor {}

struct NopLineVisitor;
impl LineVisitor for NopLineVisitor {
    fn start_general_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<NopCommandVisitor> {
        ControlFlow::Continue(NopCommandVisitor)
    }
    fn start_miscellaneous_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<NopCommandVisitor> {
        ControlFlow::Continue(NopCommandVisitor)
    }
    fn start_tool_change_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<NopCommandVisitor> {
        ControlFlow::Continue(NopCommandVisitor)
    }
}

struct NopProgramVisitor;
impl ProgramVisitor for NopProgramVisitor {
    fn start_line(&mut self, _: Span) -> ControlFlow<NopLineVisitor> {
        ControlFlow::Continue(NopLineVisitor)
    }
}

fn load_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

#[test]
fn core_parser_program_1_no_panic() {
    let src = load_fixture("program_1.gcode");
    let visitor = NopProgramVisitor;
    parse(&src, visitor);
}

#[test]
fn core_parser_program_2_no_panic() {
    let src = load_fixture("program_2.gcode");
    let visitor = NopProgramVisitor;
    parse(&src, visitor);
}

#[test]
fn core_parser_program_3_no_panic() {
    let src = load_fixture("program_3.gcode");
    let visitor = NopProgramVisitor;
    parse(&src, visitor);
}

#[test]
fn core_parser_insulpro_no_panic() {
    let src = load_fixture("Insulpro.Piping.-.115mm.OD.-.40mm.WT.txt");
    let visitor = NopProgramVisitor;
    parse(&src, visitor);
}

/// Counting visitor to assert minimum structure from fixtures.
struct CountingVisitor {
    lines: usize,
    general_codes: usize,
}

struct CountingLineVisitor<'a> {
    counts: &'a mut CountingVisitor,
}

struct CountingCommandVisitor;
impl CommandVisitor for CountingCommandVisitor {}

impl ProgramVisitor for &'_ mut CountingVisitor {
    fn start_line(&mut self, _: Span) -> ControlFlow<CountingLineVisitor<'_>> {
        self.lines += 1;
        ControlFlow::Continue(CountingLineVisitor { counts: self })
    }
}

impl LineVisitor for CountingLineVisitor<'_> {
    fn start_general_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<CountingCommandVisitor> {
        self.counts.general_codes += 1;
        ControlFlow::Continue(CountingCommandVisitor)
    }
    fn start_miscellaneous_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<CountingCommandVisitor> {
        ControlFlow::Continue(CountingCommandVisitor)
    }
    fn start_tool_change_code(
        &mut self,
        _: Number,
        _: Span,
    ) -> ControlFlow<CountingCommandVisitor> {
        ControlFlow::Continue(CountingCommandVisitor)
    }
}

#[test]
fn core_parser_program_1_structure() {
    let src = load_fixture("program_1.gcode");
    let mut visitor = CountingVisitor {
        lines: 0,
        general_codes: 0,
    };
    parse(&src, &mut visitor);
    assert!(
        visitor.lines >= 5,
        "program_1 should have at least 5 non-empty lines"
    );
    assert!(
        visitor.general_codes >= 10,
        "program_1 should have at least 10 G-codes"
    );
}

#[test]
fn core_parser_program_2_structure() {
    let src = load_fixture("program_2.gcode");
    let mut visitor = CountingVisitor {
        lines: 0,
        general_codes: 0,
    };
    parse(&src, &mut visitor);
    assert!(
        visitor.lines >= 10,
        "program_2 should have at least 10 non-empty lines"
    );
    assert!(
        visitor.general_codes >= 5,
        "program_2 should have at least 5 G-codes"
    );
}
