use core::str::FromStr;
use core::num::ParseFloatError;
use core::fmt::{self, Display, Formatter};

/// A limited-precision number where extra digits are gained by scaling an
/// integer by a constant factor. This is designed to work even for platforms
/// that may not have a built-in FPU.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Number<P: Prescalar = Thousand> {
    base: i32,
    prescalar: P,
}

impl<P: Prescalar> Number<P> {
    pub fn integral_part(&self) -> i32 {
        self.base / self.prescalar.scale() as i32
    }

    pub fn fractional_part(&self) -> u32 {
        self.base as u32 % self.prescalar.scale()
    }

    pub fn as_float(&self) -> f32 {
        self.base as f32 / self.prescalar.scale() as f32
    }

    pub fn convert<Q: Prescalar + Default>(&self) -> Number<Q> {
        let prescalar = Q::default();
        let base = (self.base * prescalar.scale() as i32) / self.prescalar.scale() as i32;

        Number { base, prescalar }
    }
}

impl<P: Prescalar+ Default> FromStr for Number<P> {
    type Err = ParseFloatError;
    
    fn from_str(s: &str) -> Result<Number<P>, Self::Err> {
        // FIXME: We really shouldn't rely on the f32 impl for this...
        Ok(Number::from(s.parse::<f32>()?))
    }
}

impl<P: Prescalar> Display for Number<P> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let integral = self.integral_part();
        let fractional = self.fractional_part();
        let precision = self.prescalar.digits();

        write!(
            f,
            "{}.{fract:0precision$}",
            integral,
            fract = fractional,
            precision = precision
        )
    }
}

macro_rules! there_and_back_again {
    ($integer_type:ty) => {
       impl<P: Prescalar + Default> From<$integer_type> for Number<P> {
           fn from(other: $integer_type) -> Number<P> {
                let prescalar = P::default();

                Number {
                    base: (other as i32).saturating_mul(prescalar.scale() as i32),
                    prescalar,
                }
           }
       }

       impl<P: Prescalar> From<Number<P>> for $integer_type {
           fn from(other: Number<P>) -> $integer_type {
               let Number { base, prescalar } = other;

               (base / prescalar.scale() as i32) as $integer_type
           }
       }
    };
    ($first:ty, $($rest:tt)*) => {
        there_and_back_again!($first);
        there_and_back_again!($($rest)*);
    }

}

there_and_back_again!(u32, i32, usize);

impl<P: Prescalar + Default> From<f32> for Number<P> {
    fn from(other: f32) -> Number<P> {
        let prescalar = P::default();

        Number {
            base: (other * prescalar.scale() as f32) as i32,
            prescalar,
        }
    }
}

impl<P: Prescalar> From<Number<P>> for f32 {
    fn from(other: Number<P>) -> f32 {
        other.as_float()
    }
}

impl<P: Prescalar + Copy> PartialEq<u32> for Number<P> {
    fn eq(&self, rhs: &u32) -> bool {
        u32::from(*self) == *rhs
    }
}

pub trait Prescalar {
    fn scale(&self) -> u32;

    fn digits(&self) -> usize {
        let mut n = self.scale();
        let mut digits = 0;

        while n >= 10 {
            digits += 1;
            n /= 10;
        }

        digits
    }
}

macro_rules! decl_prescalar {
    ($name:ident => $factor:expr;) => {
        #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
        pub struct $name;

        impl Prescalar for $name {
            fn scale(&self) -> u32 {
                $factor
            }
        }
    };
    ($name:ident => $factor:expr; $($rest:tt)*) => {
        decl_prescalar!($name => $factor;);
        decl_prescalar!($($rest)*);
    };
}

decl_prescalar! {
    Thousand => 1000;
    Hundred => 100;
    Ten => 10;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;

    #[test]
    fn count_the_digits() {
        let inputs: &[(&(Prescalar), usize)] = &[(&Thousand, 3), (&Hundred, 2), (&Ten, 1)];

        for &(prescalar, should_be) in inputs {
            let got = prescalar.digits();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn string_formatting_is_done_correctly() {
        let n: Number<Hundred> = Number::from(30);
        let should_be = "30.00";

        let got = format!("{}", n);
        assert_eq!(got, should_be);
    }

    #[test]
    fn converting_up_preserves_precision() {
        let original: Number<Ten> = Number::from(123);
        let new: Number<Thousand> = original.convert();

        assert_eq!(new.as_float(), original.as_float());
    }

    #[test]
    fn converting_down_may_lose_precision() {
        let original = 123.456;

        let number: Number<Thousand> = Number::from(original);
        assert_eq!(number.as_float(), original);

        let new: Number<Ten> = number.convert();
        assert_eq!(new.as_float(), 123.4);
    }

    #[test]
    fn convert_from_u32() {
        let n = 1234;
        let got: Number<Thousand> = n.into();

        assert_eq!(got.base, 1234 * 1000);
        assert_eq!(got.as_float(), 1234.0);
    }
}
