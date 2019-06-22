use crate::operations::{ConversionError, Op, Operation};
use crate::State;
use gcode::{Gcode, Parser, Span, TokenKind};
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;
use uom::num::Zero;
use uom::si::f32::*;

#[derive(Debug)]
pub struct Simulator {
    mapping: OperationMap,
}

impl Simulator {
    pub fn load(src: &str) -> Result<Simulator, ParseErrors> {
        let mut cb = ParseErrors::default();
        let commands: Vec<_> =
            Parser::new_with_callbacks(src, &mut cb).collect();

        if cb.has_errors() {
            return Err(cb);
        }

        unimplemented!()
    }
}

#[derive(Default)]
pub struct ParseErrors {
    pub unexpected: Vec<UnexpectedToken>,
    pub unexpected_eof: Option<UnexpectedEOF>,
    pub mangled_inputs: Vec<MangledInput>,
}

impl ParseErrors {
    fn has_errors(&self) -> bool {
        self.unexpected.len() > 0
            || self.unexpected_eof.is_some()
            || self.mangled_inputs.len() > 0
    }
}

impl gcode::Callbacks for ParseErrors {
    fn unexpected_token(
        &mut self,
        found: TokenKind,
        span: Span,
        expected: &[TokenKind],
    ) {
        self.unexpected.push(UnexpectedToken {
            found,
            span,
            expected: expected.to_vec(),
        });
    }

    fn unexpected_eof(&mut self, expected: &[TokenKind]) {
        self.unexpected_eof = Some(UnexpectedEOF {
            expected: expected.to_vec(),
        });
    }

    fn mangled_input(&mut self, input: &str, span: Span) {
        self.mangled_inputs.push(MangledInput {
            input: input.to_string(),
            span,
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnexpectedToken {
    pub found: TokenKind,
    pub span: Span,
    pub expected: Vec<TokenKind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnexpectedEOF {
    pub expected: Vec<TokenKind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MangledInput {
    pub input: String,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Simulation {
    initial_states: Vec<(Time, Id<Op>, State)>,
    total_time: Time,
}

/// A mapping from raw `Gcode`s to their corresponding operations.
#[derive(Debug, Default)]
pub struct OperationMap {
    pub gcodes: Arena<Gcode>,
    pub operations: Arena<Op>,

    pub op_order: Vec<Id<Op>>,
    pub gcodes_by_span: HashMap<Span, Id<Gcode>>,
    pub op_by_gcode_id: HashMap<Id<Op>, Id<Gcode>>,
    pub cant_parse: Vec<(Id<Gcode>, ConversionError)>,
}

impl OperationMap {
    fn add_one(&mut self, gcode: Gcode) {
        let id = self.gcodes.alloc(gcode);
        let gcode = &self.gcodes[id];

        self.gcodes_by_span.insert(gcode.span(), id);

        match Op::try_from(gcode) {
            Ok(op) => {
                let op_id = self.operations.alloc(op);
                self.op_by_gcode_id.insert(op_id, id);
            }
            Err(e) => self.cant_parse.push((id, e)),
        }
    }
}

impl FromIterator<Gcode> for OperationMap {
    fn from_iter<I: IntoIterator<Item = Gcode>>(iter: I) -> OperationMap {
        let mut sim = OperationMap::default();

        for command in iter {
            sim.add_one(command);
        }

        sim
    }
}
