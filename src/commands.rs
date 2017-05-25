#[derive(Clone, PartialEq, Debug)]
pub enum G {
    G00(Point),
}

impl G {
    /// Get a short description of this G code.
    ///
    /// From: https://www.tormach.com/g_code_table.html
    fn description(&self) -> &'static str {
        match *self {
            G::G00(_) => "Rapid Positioning",
        }
    }

    fn set_arg(&mut self, _arg: Argument) {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}
