//! Functionality for working with decimal numbers that have a fixed precision.

#![allow(missing_docs)]

use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};
use core::marker::PhantomData;
use core::ops::{Add, Div, Mul, Sub};
use core::str::FromStr;

/// A fixed-precision decimal number.
///
/// ```rust
/// # use gcode::prescaled::{Prescaled, Thousand, Scalar};
/// let original: f64 = 3.140;
/// let prescaled: Prescaled<Thousand> = original.into();
///
/// assert_eq!(prescaled.integral_part(), 3);
/// assert_eq!(prescaled.fractional_part(), 140);
/// let int = prescaled.integral_part() as f64;
/// let fract = prescaled.fractional_part() as f64;
/// let scale = Thousand::SCALE as f64;
/// assert_eq!(int + fract / scale, original);
/// ```
#[derive(Debug)]
pub struct Prescaled<S>(i64, PhantomData<S>);

impl<S: Scalar> Prescaled<S> {
    pub(crate) fn new(integral: i64, fractional: i64) -> Prescaled<S> {
        debug_assert!(
            integral
                .checked_mul(S::SCALE)
                .and_then(|n| n.checked_add(fractional))
                .is_some(),
            "Prescaling ({}, {}) would cause an overflow",
            integral,
            fractional,
        );

        Prescaled(integral * S::SCALE + fractional, PhantomData)
    }

    pub fn integral_part(&self) -> i64 {
        self.0 / S::SCALE
    }

    pub fn fractional_part(&self) -> i64 {
        self.0 % S::SCALE
    }
}

impl<S> Default for Prescaled<S> {
    fn default() -> Prescaled<S> {
        Prescaled(0, PhantomData)
    }
}

impl<S: Scalar> From<Prescaled<S>> for f64 {
    fn from(other: Prescaled<S>) -> f64 {
        other.0 as f64 / S::SCALE as f64
    }
}

impl<S: Scalar> From<f64> for Prescaled<S> {
    fn from(other: f64) -> Prescaled<S> {
        let integral = other.trunc() as i64;
        let fractional = (other.fract() * S::SCALE as f64) as i64;

        Prescaled::new(integral, fractional)
    }
}

impl<S> Copy for Prescaled<S> {}
impl<S> Eq for Prescaled<S> {}
impl<S> PartialEq for Prescaled<S> {
    fn eq(&self, other: &Prescaled<S>) -> bool {
        self.0 == other.0
    }
}
impl<S: Scalar> PartialEq<f64> for Prescaled<S> {
    fn eq(&self, other: &f64) -> bool {
        f64::from(*self) == *other
    }
}
impl<S> Clone for Prescaled<S> {
    fn clone(&self) -> Prescaled<S> {
        Prescaled(self.0, PhantomData)
    }
}
impl<S> PartialOrd for Prescaled<S> {
    fn partial_cmp(&self, other: &Prescaled<S>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<S> Ord for Prescaled<S> {
    fn cmp(&self, other: &Prescaled<S>) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl<S: Scalar> Display for Prescaled<S> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl<S> Add for Prescaled<S> {
    type Output = Prescaled<S>;
    fn add(self, other: Prescaled<S>) -> Prescaled<S> {
        Prescaled(self.0 + other.0, PhantomData)
    }
}

impl<S: Scalar> Add<f64> for Prescaled<S> {
    type Output = Prescaled<S>;
    fn add(self, other: f64) -> Prescaled<S> {
        self + Self::from(other)
    }
}

impl<S> Sub for Prescaled<S> {
    type Output = Prescaled<S>;
    fn sub(self, other: Prescaled<S>) -> Prescaled<S> {
        Prescaled(self.0 - other.0, PhantomData)
    }
}

impl<S: Scalar> Sub<f64> for Prescaled<S> {
    type Output = Prescaled<S>;
    fn sub(self, other: f64) -> Prescaled<S> {
        self - Self::from(other)
    }
}
impl<S: Scalar> Mul for Prescaled<S> {
    type Output = Prescaled<S>;
    fn mul(self, other: Prescaled<S>) -> Prescaled<S> {
        Prescaled((self.0 * other.0) / S::SCALE, PhantomData)
    }
}

impl<S: Scalar> Mul<f64> for Prescaled<S> {
    type Output = Prescaled<S>;
    fn mul(self, other: f64) -> Prescaled<S> {
        self * Self::from(other)
    }
}

impl<S: Scalar> Div for Prescaled<S> {
    type Output = Prescaled<S>;
    fn div(self, other: Prescaled<S>) -> Prescaled<S> {
        Prescaled(self.0 * S::SCALE / other.0, PhantomData)
    }
}

impl<S: Scalar> Div<f64> for Prescaled<S> {
    type Output = Prescaled<S>;
    fn div(self, other: f64) -> Prescaled<S> {
        self / Self::from(other)
    }
}

impl<S: Scalar> FromStr for Prescaled<S> {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Prescaled<S>, Self::Err> {
        let original: f64 = s.parse()?;
        Ok(original.into())
    }
}

/// A type-level scalar number.
pub trait Scalar {
    /// The whole point.
    const SCALE: i64;
}

macro_rules! scalar {
    ($name:ident, $scale:expr, $docs:expr) => {
        #[doc = $docs]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub enum $name {}

        impl Scalar for $name {
            const SCALE: i64 = $scale;
        }
    };
    ( $($name:ident = $scale:expr ;)*) => {
        $( scalar!($name, $scale, concat!( "A type-level integer representing ", stringify!($scale), ".")); )*
    }
}

scalar! {
    One = 1;
    Ten = 10;
    Hundred = 100;
    Thousand = 1000;
    TenThousand = 10_000;
    HundredThousand = 100_000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_a_normal_integer() {
        let number: f64 = -3.14;

        let prescaled: Prescaled<Hundred> = number.into();
        assert_eq!(prescaled.0, -314);

        let round_tripped: f64 = prescaled.into();
        assert_eq!(number, round_tripped);
    }

    #[test]
    fn too_many_digits_loses_precision_silently() {
        let number: f64 = 3.14159;

        let prescaled: Prescaled<Hundred> = number.into();
        assert_eq!(prescaled.0, 314);
        assert_eq!(3.14, f64::from(prescaled));
    }

    #[test]
    fn parsing_works_as_normal() {
        let src = "3.1415";

        let got: Prescaled<TenThousand> = src.parse().unwrap();
        assert_eq!(got, 3.1415);

        let rounded: Prescaled<Hundred> = src.parse().unwrap();
        assert_eq!(rounded, 3.14);
    }

    /// A `Prescaled<T>` version of `assert_eq!()` which takes into account
    /// rounding and only makes sure there's an error of less than 1 in the
    /// last place.
    macro_rules! pretty_close {
        ($left:expr, $right:expr) => {{
            fn get_scale<S: Scalar>(_: Prescaled<S>) -> i64 {
                S::SCALE
            }
            let left: Prescaled<_> = $left;
            let scale = get_scale(left) as f64;
            let diff = f64::from(left) - f64::from($right);
            let error_in_last_place = diff * scale;
            assert!(
                error_in_last_place.abs() < 1.0,
                "{} != {} (error is {})",
                $left,
                $right,
                error_in_last_place / scale
            );
        }};
    }

    #[test]
    fn normal_binary_operations() {
        let original: f64 = 3.1415;
        let prescaled: Prescaled<TenThousand> = original.into();

        assert_eq!(prescaled + prescaled, original + original);
        pretty_close!(prescaled + 2.0, original + 2.0);
        pretty_close!(prescaled * prescaled, original * original);
        pretty_close!(prescaled * 2.0, original * 2.0);
        pretty_close!(prescaled / prescaled, 1.0);
        pretty_close!(prescaled / 2.0, original / 2.0);
    }
}
