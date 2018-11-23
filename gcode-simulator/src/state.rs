//! Machine state.

use uom::si::f32::*;
use uom::si::length::{inch, millimeter};
use uom::si::time::second;
use uom::si::velocity::millimeter_per_second;

/// The internal state of a simple 2-dimensional gantry system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct State {
    pub current_position: AxisPositions,
    pub feed_rate: Velocity,
    pub coordinate_mode: CoordinateMode,
    pub units: Units,
}

impl State {
    /// Convert the value to a length, taking into account the current unit
    /// system.
    pub fn to_length(&self, value: f32) -> Length {
        match self.units {
            Units::Metric => Length::new::<millimeter>(value),
            Units::Imperial => Length::new::<inch>(value),
        }
    }

    /// Convert the value to a speed, taking into account the current unit
    /// system.
    pub fn to_speed(&self, value: f32) -> Velocity {
        match self.units {
            Units::Metric => Velocity::new::<millimeter_per_second>(value),
            Units::Imperial => {
                Length::new::<inch>(value) / Time::new::<second>(1.0)
            }
        }
    }

    pub fn with_current_position(mut self, value: AxisPositions) -> Self {
        self.current_position = value;
        self
    }

    pub fn with_x(mut self, value: f32) -> Self {
        self.current_position = AxisPositions {
            x: self.to_length(value),
            ..self.current_position
        };
        self
    }

    pub fn with_y(mut self, value: f32) -> Self {
        self.current_position = AxisPositions {
            y: self.to_length(value),
            ..self.current_position
        };
        self
    }

    pub fn with_feed_rate(mut self, value: f32) -> Self {
        self.feed_rate = self.to_speed(value);
        self
    }

    pub fn with_coordinate_mode(mut self, value: CoordinateMode) -> Self {
        self.coordinate_mode = value;
        self
    }

    pub fn with_units(mut self, value: Units) -> Self {
        self.units = value;
        self
    }
}

impl Default for State {
    fn default() -> State {
        State {
            current_position: AxisPositions::default(),
            feed_rate: Velocity::new::<millimeter_per_second>(100.0),
            coordinate_mode: CoordinateMode::Absolute,
            units: Units::Metric,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoordinateMode {
    Absolute,
    Relative,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Units {
    Metric,
    Imperial,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AxisPositions {
    pub x: Length,
    pub y: Length,
}

impl Default for AxisPositions {
    fn default() -> AxisPositions {
        AxisPositions {
            x: Length::new::<millimeter>(0.0),
            y: Length::new::<millimeter>(0.0),
        }
    }
}
