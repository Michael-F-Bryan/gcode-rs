use core::fmt::{self, Formatter, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gcode {
    pub mnemonic: Mnemonic,
    pub number: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mnemonic {
    /// A program number (`O555`).
    ProgramNumber,
    /// A tool change command (`T6`).
    ToolChange,
    /// A machine-specific routine (`M3`).
    MachineRoutine,
    /// A general command (`G01`).
    General,
}

/// A limited-precision number where extra digits are gained by scaling an
/// integer by a constant factor.
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
}

impl<P: Prescalar> Display for Number<P> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let integral = self.integral_part();
        let fractional = self.fractional_part();
        let precision = self.prescalar.digits();

        write!(f, "{}.{fract:0precision$}", integral, fract = fractional, precision = precision)
    }
}

impl<P: Prescalar + Default> From<u32> for Number<P> {
    fn from(other: u32) -> Number<P> {
        let prescalar = P::default();

        Number {
            base: other.saturating_mul(prescalar.scale()) as i32,
            prescalar,
        }
    }
}

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
    ($name:ident, $factor:expr) => {
        #[derive(Debug, Copy, Clone, Default, PartialEq)]
        pub struct $name;

        impl Prescalar for $name {
            fn scale(&self) -> u32 {
                $factor
            }
        }
    }
}

macro_rules! decl_prescalars {
    ($name:ident, $factor:expr; $($rest:tt)*) => {
        decl_prescalar!($name, $factor);

        decl_prescalars!($($rest)*);
    };
    () => {}
}

decl_prescalars! {
    Thousand, 1000;
    Hundred, 100;
    Ten, 10;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;

    #[test]
    fn count_the_digits() {
        let inputs: &[(&(Prescalar), usize)] = &[
            (&Thousand, 3),
            (&Hundred, 2),
            (&Ten, 1),
        ];

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
}
