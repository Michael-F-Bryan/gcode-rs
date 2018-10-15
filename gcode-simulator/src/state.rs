/// The internal state of a simple 2-dimensional gantry system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct State {
    pub x: f32,
    pub y: f32,
    /// The feed rate in units per minute.
    pub feed_rate: f32,
    pub coordinate_mode: CoordinateMode,
}

impl Default for State {
    fn default() -> State {
        State {
            x: 0.0,
            y: 0.0,
            feed_rate: 100.0,
            coordinate_mode: CoordinateMode::Absolute,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoordinateMode {
    Absolute,
    Relative,
}
