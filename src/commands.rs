#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct G {
    pub code: u32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub feed_rate: Option<f32>,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Argument {
    X(f32),
    Y(f32),
    Z(f32),
    Feed(f32),

    /// A hidden variant that nobody can access, for future proofing.
    #[doc(hidden)]
    _Nonexhaustive,
}


impl From<u32> for G {
    fn from(other: u32) -> Self {
        G {
            code: other,
            ..Default::default()
        }
    }
}
