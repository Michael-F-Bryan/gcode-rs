use crate::types::{Argument, Gcode, Mnemonic};

/// A generic predicate.
pub trait Predicate<T: ?Sized> {
    fn evaluate(&mut self, item: &T) -> bool;

    /// Create a new [`Predicate<T>`] which is effectively
    /// `this.evaluate(item) && other.evaluate(item)`.
    fn and<P>(self, other: P) -> And<Self, P>
    where
        P: Predicate<T>,
        Self: Sized,
    {
        And {
            left: self,
            right: other,
        }
    }

    /// Create a new [`Predicate<T>`] which is effectively
    /// `this.evaluate(item) || other.evaluate(item)`.
    fn or<P>(self, other: P) -> Or<Self, P>
    where
        P: Predicate<T>,
        Self: Sized,
    {
        Or {
            left: self,
            right: other,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct And<L, R> {
    left: L,
    right: R,
}

impl<T, L, R> Predicate<T> for And<L, R>
where
    L: Predicate<T>,
    R: Predicate<T>,
    T: ?Sized,
{
    fn evaluate(&mut self, item: &T) -> bool {
        self.left.evaluate(item) && self.right.evaluate(item)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Or<L, R> {
    left: L,
    right: R,
}

impl<T, L, R> Predicate<T> for Or<L, R>
where
    L: Predicate<T>,
    R: Predicate<T>,
    T: ?Sized,
{
    fn evaluate(&mut self, item: &T) -> bool {
        self.left.evaluate(item) || self.right.evaluate(item)
    }
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

impl Predicate<Argument> for char {
    fn evaluate(&mut self, item: &Argument) -> bool {
        item.letter == *self
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
