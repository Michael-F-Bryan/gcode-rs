use crate::state::CoordinateMode;

singleton_cmd! {
    /// Tell the machine to use absolute coordinates.
    AbsoluteCoordinates, 90, |state| state.with_coordinate_mode(CoordinateMode::Absolute)
}

singleton_cmd! {
    /// Tell the machine to use relative coordinates.
    RelativeCoordinates, 91, |state| state.with_coordinate_mode(CoordinateMode::Relative)
}
