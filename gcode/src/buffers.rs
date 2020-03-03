use crate::{Comment, GCode, Word};
use arrayvec::{Array, ArrayVec};
use core::fmt::{self, Debug, Display, Formatter};

#[cfg(not(feature = "std"))]
pub type DefaultBuffers = SmallFixedBuffers;

#[cfg(feature = "std")]
pub type DefaultBuffers = VecBuffers;

/// A set of type aliases defining the types to use when storing data.
pub trait Buffers<'input> {
    type Arguments: Buffer<Word> + Default;
    type Commands: Buffer<GCode<Self::Arguments>> + Default;
    type Comments: Buffer<Comment<'input>> + Default;
}

pub trait Buffer<T> {
    fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>>;

    fn as_slice(&self) -> &[T];
}

impl<T, A: Array<Item = T>> Buffer<T> for ArrayVec<A> {
    fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>> {
        ArrayVec::try_push(self, item).map_err(|e| CapacityError(e.element()))
    }

    fn as_slice(&self) -> &[T] { &self }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SmallFixedBuffers {}

impl<'input> Buffers<'input> for SmallFixedBuffers {
    type Arguments = ArrayVec<[Word; 5]>;
    type Commands = ArrayVec<[GCode<Self::Arguments>; 1]>;
    type Comments = ArrayVec<[Comment<'input>; 1]>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg(feature = "std")]
pub enum VecBuffers {}

#[cfg(feature = "std")]
impl<'input> Buffers<'input> for VecBuffers {
    type Arguments = Vec<Word>;
    type Commands = Vec<GCode<Self::Arguments>>;
    type Comments = Vec<Comment<'input>>;
}

#[cfg(feature = "std")]
impl<T> Buffer<T> for Vec<T> {
    fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>> {
        self.push(item);
        Ok(())
    }

    fn as_slice(&self) -> &[T] { &self }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CapacityError<T>(pub T);

impl<T: Debug> Display for CapacityError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "insufficient capacity")
    }
}

#[cfg(feature = "std")]
impl<T: Debug> std::error::Error for CapacityError<T> {}
