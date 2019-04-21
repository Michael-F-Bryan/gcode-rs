use crate::operations::{ConversionError, Op, Operation};
use crate::State;
use gcode::{Gcode, Span};
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;
use uom::num::Zero;
use uom::si::f32::*;

pub fn simulate_motion(ops: &OperationMap) -> Simulation {
    let mut sim = Simulation::default();
    let mut state = State::default();

    for &op_id in &ops.op_order {
        sim.initial_states
            .push((sim.total_time, op_id, state.clone()));

        let op = &ops.operations[op_id];
        let duration = op.duration(&state);

        if duration >= Time::zero() {
            state = op.state_after(duration, state);
        }

        sim.total_time += duration;
    }

    sim
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
