/// The internal state of a simple 2-dimensional gantry system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct State {
    pub x: f32,
    pub y: f32,
    pub feed_rate: f32,
    pub coordinate_mode: CoordinateMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoordinateMode {
    Absolute,
    Relative,
}
