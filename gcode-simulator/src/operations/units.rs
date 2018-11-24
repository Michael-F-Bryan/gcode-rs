use crate::state::Units;

singleton_cmd! {
    /// Use the imperial units system.
    Imperial, 20, |state| state.with_units(Units::Imperial)
}

singleton_cmd! {
    /// Use the metric units system.
    Metric, 21, |state| state.with_units(Units::Imperial)
}
