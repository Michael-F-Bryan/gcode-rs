extern crate gcode;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct State {}

pub trait Operation {
    fn state_after(&self, seconds: f32, initial_state: State) -> State;
    fn duration(&self) -> f32;
}
