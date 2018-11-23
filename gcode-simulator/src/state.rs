//! Machine state.

#[cfg(test)]
use approx::{AbsDiffEq, RelativeEq};
use core::ops::{Add, Mul, Sub};
#[allow(unused_imports)]
use libm::F32Ext;
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

/// A set of positions representing the current location of each axis.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AxisPositions {
    pub x: Length,
    pub y: Length,
}

impl AxisPositions {
    pub fn length(&self) -> Length {
        let AxisPositions { x, y } = *self;
        let squares = x * x + y * y;

        Length {
            value: squares.value.sqrt(),
            ..x
        }
    }
}

impl Default for AxisPositions {
    fn default() -> AxisPositions {
        AxisPositions {
            x: Length::new::<millimeter>(0.0),
            y: Length::new::<millimeter>(0.0),
        }
    }
}

impl Add for AxisPositions {
    type Output = AxisPositions;

    fn add(self, other: AxisPositions) -> AxisPositions {
        AxisPositions {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for AxisPositions {
    type Output = AxisPositions;

    fn sub(self, other: AxisPositions) -> AxisPositions {
        AxisPositions {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for AxisPositions {
    type Output = AxisPositions;

    fn mul(self, other: f32) -> AxisPositions {
        AxisPositions {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

#[cfg(test)]
impl AbsDiffEq for AxisPositions {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        <f32 as AbsDiffEq>::default_epsilon()
    }

    fn abs_diff_eq(
        &self,
        other: &AxisPositions,
        epsilon: Self::Epsilon,
    ) -> bool {
        let AxisPositions { x, y } = *self;

        x.value.abs_diff_eq(&other.x.value, epsilon)
            && y.value.abs_diff_eq(&other.y.value, epsilon)
    }
}

#[cfg(test)]
impl RelativeEq for AxisPositions {
    fn default_max_relative() -> Self::Epsilon {
        <f32 as RelativeEq>::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &AxisPositions,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        let AxisPositions { x, y } = *self;

        x.value.relative_eq(&other.x.value, epsilon, max_relative)
            && y.value.relative_eq(&other.y.value, epsilon, max_relative)
    }
}
