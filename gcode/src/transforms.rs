use crate::types::{Gcode, Mnemonic};

/// A helper trait which adds useful extension methods to all [`Gcode`]
/// iterators.
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
}

impl<I> GcodeTransforms for I where I: Iterator<Item = Gcode> {}

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

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        if self.selector.evaluate(&next) {
            Some((self.map)(next))
        } else {
            Some(next)
        }
    }
}

/// A generic predicate.
pub trait Predicate<T: ?Sized> {
    fn evaluate(&mut self, item: &T) -> bool;
}

impl<F, T> Predicate<T> for F
where
    F: FnMut(&T) -> bool,
{
    fn evaluate(&mut self, item: &T) -> bool {
        (self)(item)
    }
}

impl Predicate<Gcode> for char {
    /// Select any [`Gcode`]s which have this argument.
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.args().iter().any(|arg| arg.letter == *self)
    }
}

impl Predicate<Gcode> for usize {
    /// Select any [`Gcode`]s with this major number.
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.major_number() == *self
    }
}

impl Predicate<Gcode> for Mnemonic {
    /// Select any [`Gcode`]s which have this [`Mnemonic`].
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.mnemonic() == *self
    }
}

impl<'a, T, P> Predicate<T> for &'a mut [P]
where
    P: Predicate<T>,
{
    fn evaluate(&mut self, item: &T) -> bool {
        self.iter_mut().any(|pred| pred.evaluate(item))
    }
}

macro_rules! array_predicate {
    ($($count:expr),*) => {
        $(
            impl<T, P> Predicate<T> for [P; $count]
            where
                P: Predicate<T>,
            {
                fn evaluate(&mut self, item: &T) -> bool {
                    let mut slice: &mut [P] = &mut *self;
                    slice.evaluate(item)
                }
            }
        )*
    };
}

array_predicate!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

#[cfg(test)]
mod tests {
    use super::*;
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
}
