use crate::types::{Argument, Gcode, Mnemonic};

/// A generic predicate.
///
/// The intention behind this helper trait is to make applying transformations
/// to a subset of inputs more ergonomic.
///
/// ```rust
/// use gcode::{Argument, Gcode, Mnemonic, Span};
/// use gcode::transforms::Predicate;
///
/// let g90 = Gcode::new(Mnemonic::General, 90.0);
/// let g00 = Gcode::new(Mnemonic::General, 0.0)
///     .with_argument(Argument::new('X', 500.0));
/// let m6 = Gcode::new(Mnemonic::Miscellaneous, 6.0);
/// let m0 = Gcode::new(Mnemonic::Miscellaneous, 0.0);
///
/// // mnemonics can be used to match any gcode with that mnemonic
/// let mut matches_g = Mnemonic::General;
/// assert!(matches_g.evaluate(&g90));
/// assert!(!matches_g.evaluate(&m6));
///
/// // You can also match on a major number
/// let mut matches_0 = 0;
/// assert!(matches_0.evaluate(&g00));
/// assert!(matches_0.evaluate(&m0));
/// assert!(!matches_0.evaluate(&g90));
///
/// // predicates can also be combined to make more specific predicates
/// let mut matches_g90 = Mnemonic::General.and(90);
/// assert!(matches_g90.evaluate(&g90));
/// assert!(!matches_g90.evaluate(&m0));
/// assert!(!matches_g90.evaluate(&g00));
///
/// // you can also combine mutation and closures in interesting ways
/// let mut count = 0;
/// let mut matches_every_other_call = |_: &Gcode| {
///     count += 1;
///     count % 2 == 0
/// };
/// assert!(!matches_every_other_call.evaluate(&g00));
/// assert!(matches_every_other_call.evaluate(&g90));
/// assert!(!matches_every_other_call.evaluate(&g90));
/// assert!(matches_every_other_call.evaluate(&g00));
///
/// // What about only matching on a G-code with an `X` parameter?
/// let mut x = 'X';
/// assert!(x.evaluate(&g00));
/// ```
pub trait Predicate<T: ?Sized> {
    /// Evaluate the predicate based on the provided input.
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

/// See [`Predicate::and()`] for more.
///
/// [`Predicate::and()`]: ./trait.Predicate.html#method.and
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

/// See [`Predicate::or()`] for more.
///
/// [`Predicate::or()`]: ./trait.Predicate.html#method.or
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
    #[inline]
    fn evaluate(&mut self, item: &T) -> bool {
        self.left.evaluate(item) || self.right.evaluate(item)
    }
}

impl<F, T> Predicate<T> for F
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn evaluate(&mut self, item: &T) -> bool {
        (self)(item)
    }
}

impl Predicate<Gcode> for char {
    /// Select any [`Gcode`]s which have this argument.
    #[inline]
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.args().iter().any(|arg| arg.letter == *self)
    }
}

impl Predicate<Argument> for char {
    #[inline]
    fn evaluate(&mut self, item: &Argument) -> bool {
        item.letter == *self
    }
}

impl Predicate<Gcode> for usize {
    /// Select any [`Gcode`]s with this major number.
    #[inline]
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.major_number() == *self
    }
}

impl Predicate<Gcode> for Mnemonic {
    /// Select any [`Gcode`]s which have this [`Mnemonic`].
    #[inline]
    fn evaluate(&mut self, item: &Gcode) -> bool {
        item.mnemonic() == *self
    }
}

impl<'a, T, P> Predicate<T> for &'a mut [P]
where
    P: Predicate<T>,
{
    #[inline]
    fn evaluate(&mut self, item: &T) -> bool {
        self.iter_mut().any(|pred| pred.evaluate(item))
    }
}

macro_rules! array_predicate {
    ($($count:expr),*) => {
        $(
            #[doc(hidden)]
            impl<T, P> Predicate<T> for [P; $count]
            where
                P: Predicate<T>,
            {
                #[inline]
                fn evaluate(&mut self, item: &T) -> bool {
                    let mut slice: &mut [P] = &mut *self;
                    slice.evaluate(item)
                }
            }
        )*
    };
}

// Feel free to make a PR if you need to support larger arrays
array_predicate!(
    0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31
);
