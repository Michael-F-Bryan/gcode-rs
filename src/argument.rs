#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Argument {
    X(f64),
    Y(f64),
    Z(f64),
    Feed(f64),

    /// A hidden variant that nobody can access, for future proofing.
    #[doc(hidden)]
    _Nonexhaustive,
}
