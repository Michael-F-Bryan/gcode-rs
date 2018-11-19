//! Functionality for transforming gcodes.

mod predicate;

pub use self::predicate::{And, Or, Predicate};

use crate::types::{Argument, Gcode};

/// A helper trait which adds useful extension methods to all [`Gcode`]
/// iterators.
///
/// [`Gcode`]: ../struct.Gcode.html
pub trait GcodeTransforms: Iterator<Item = Gcode> {
    /// Apply a transformation to all [`Gcode`]s which satisfy a particular
    /// condition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gcode::{Argument, GcodeTransforms, Span};
    ///
    /// let f = Argument {
    ///     letter: 'F',
    ///     value: 10_000.0,
    ///     span: Span::placeholder(),
    /// };
    ///
    /// // append the `f` argument to all gcodes with a major number of 0 or 1.
    /// let gcodes = gcode::parse("G90 G00 X5.0 Y-2.2 G01 X10.0 Z-2.0")
    ///     .map_gcode([0, 1], |gcode| gcode.with_argument(f));
    ///
    /// for gcode in gcodes {
    ///     let major = gcode.major_number();
    ///     if major == 0 || major == 1 {
    ///         assert_eq!(gcode.value_for('F'), Some(10_000.0));
    ///     }
    /// }
    /// ```
    ///
    /// [`Gcode`]: ../struct.Gcode.html
    fn map_gcode<S, F>(self, which_gcode: S, map: F) -> Map<Self, S, F>
    where
        S: Predicate<Gcode>,
        F: FnMut(Gcode) -> Gcode,
        Self: Sized,
    {
        Map {
            iter: self,
            selector: which_gcode,
            map,
        }
    }

    /// Change the value of a particular gcode's argument.
    ///
    /// # Examples
    ///
    /// To apply a translation in the `X` direction to a selection of move
    /// commands (e.g. `G00`, `G01`, `G02`, and `G03`) you can do the following:
    ///
    /// ```rust
    /// use gcode::GcodeTransforms;
    ///
    /// let src = "G90 G00 X5.0 Y-2.2 G01 X10.0 Z-2.0 G04 X30.0";
    /// let delta = 50.0;
    ///
    /// let translated = gcode::parse(src)
    ///     .map_argument([0, 1, 2, 3], 'X', |x| x + delta);
    ///
    /// let x_values_should_be = vec![5.0 + delta, 10.0 + delta];
    ///
    /// let got: Vec<f32> = translated
    ///     .filter(|gcode| [0, 1, 2, 3].contains(&gcode.major_number()))
    ///     .filter_map(|gcode| gcode.value_for('X'))
    ///     .collect();
    ///
    /// assert_eq!(got, x_values_should_be);
    /// ```
    fn map_argument<S, A, F>(
        self,
        which_gcode: S,
        which_arg: A,
        map: F,
    ) -> MapArg<Self, S, A, F>
    where
        Self: Sized,
        S: Predicate<Gcode>,
        A: Predicate<Argument>,
        F: FnMut(f32) -> f32,
    {
        MapArg {
            iter: self,
            gcode_selector: which_gcode,
            arg_selector: which_arg,
            map,
        }
    }
}

impl<I> GcodeTransforms for I where I: Iterator<Item = Gcode> {}

/// The return type from [`GcodeTransforms::map_gcode()`].
///
/// [`GcodeTransforms::map_gcode()`]: trait.GcodeTransforms.html#method.map_gcode
#[derive(Debug)]
pub struct Map<I, S, F> {
    iter: I,
    selector: S,
    map: F,
}

impl<I, S, F> Iterator for Map<I, S, F>
where
    I: Iterator<Item = Gcode>,
    S: Predicate<Gcode>,
    F: FnMut(Gcode) -> Gcode,
{
    type Item = Gcode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        if self.selector.evaluate(&next) {
            Some((self.map)(next))
        } else {
            Some(next)
        }
    }
}

/// The return type from [`GcodeTransforms::map_argument()`].
///
/// [`GcodeTransforms::map_argument()`]: trait.GcodeTransforms.html#method.map_argument
#[derive(Debug)]
pub struct MapArg<I, S, A, F> {
    iter: I,
    gcode_selector: S,
    arg_selector: A,
    map: F,
}

impl<I, S, A, F> Iterator for MapArg<I, S, A, F>
where
    I: Iterator<Item = Gcode>,
    S: Predicate<Gcode>,
    A: Predicate<Argument>,
    F: FnMut(f32) -> f32,
{
    type Item = Gcode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.iter.next()?;

        if self.gcode_selector.evaluate(&next) {
            for arg in next.args_mut() {
                if self.arg_selector.evaluate(&*arg) {
                    arg.value = (self.map)(arg.value);
                }
            }
        }

        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Mnemonic, Span};
    use std::prelude::v1::*;

    #[test]
    fn no_op() {
        let src = "G90 G00 X5.0 Y-2.2 G01 X10.0 Z-2.0";
        let should_be: Vec<_> = crate::parse(src).collect();

        let transformed: Vec<_> = crate::parse(src)
            .map_gcode(|_: &Gcode| true, |gcode| gcode)
            .collect();

        assert_eq!(transformed, should_be);
    }

    #[test]
    fn translate_x_for_motion_commands() {
        let src = "G90 G00 X50.0 G01 X-2.5 Y3.14 G4 X 1.0";
        let delta = 10.0;

        let should_be = vec![
            Gcode::new(Mnemonic::General, 90.0).with_span(Span::new(0, 4, 0)),
            Gcode::new(Mnemonic::General, 0.0)
                .with_span(Span::new(4, 14, 0))
                .with_argument(Argument {
                    letter: 'X',
                    value: 50.0 + delta,
                    span: Span::new(8, 14, 0),
                }),
            Gcode::new(Mnemonic::General, 1.0)
                .with_span(Span::new(14, 30, 0))
                .with_argument(Argument::new(
                    'X',
                    -2.5 + delta,
                    Span::new(18, 24, 0),
                ))
                .with_argument(Argument::new('Y', 3.14, Span::new(24, 30, 0))),
            // Add a gcode with an X argument we want to ignore
            Gcode::new(Mnemonic::General, 4.0)
                .with_span(Span::new(30, 38, 0))
                .with_argument(Argument::new('X', 1.0, Span::new(33, 38, 0))),
        ];

        let got: Vec<_> = crate::parse(src)
            .map_argument([0, 1], 'X', |x| x + delta)
            .collect();

        assert_eq!(got, should_be);
    }
}
