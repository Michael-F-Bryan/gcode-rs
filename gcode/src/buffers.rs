//! Buffer Management.
//!
//! This module is mainly intended for use cases when the amount of space that
//! can be consumed by buffers needs to be defined at compile time. For most
//! users, the [`DefaultBuffers`] alias should be suitable.

use crate::{Comment, GCode, Word};
use arrayvec::{Array, ArrayVec};
use core::fmt::{self, Debug, Display, Formatter};

/// The default buffer type for this platform.
/// 
/// This is a type alias for [`SmallFixedBuffers`] because the crate is compiled 
/// without the *"std"* feature.
#[cfg(not(feature = "std"))]
pub type DefaultBuffers = SmallFixedBuffers;

/// The default buffer type for this platform.
/// 
/// This is a type alias for [`VecBuffers`] because the crate is compiled 
/// with the *"std"* feature.
#[cfg(feature = "std")]
pub type DefaultBuffers = VecBuffers;

/// A set of type aliases defining the types to use when storing data.
pub trait Buffers<'input> {
    type Arguments: Buffer<Word> + Default;
    type Commands: Buffer<GCode<Self::Arguments>> + Default;
    type Comments: Buffer<Comment<'input>> + Default;
}

/// Something which can store items sequentially in memory. This doesn't 
/// necessarily require dynamic memory allocation.
pub trait Buffer<T> {
    /// Try to add another item to this [`Buffer`], returning the item if there
    /// is no more room.
    fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>>;

    /// The items currently stored in the [`Buffer`].
    fn as_slice(&self) -> &[T];
}

impl<T, A: Array<Item = T>> Buffer<T> for ArrayVec<A> {
    fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>> {
        ArrayVec::try_push(self, item).map_err(|e| CapacityError(e.element()))
    }

    fn as_slice(&self) -> &[T] { &self }
}

/// The smallest usable set of [`Buffers`].
///
/// ```rust
/// # use gcode::{Line, GCode, buffers::{Buffers, SmallFixedBuffers}};
/// let line_size = std::mem::size_of::<Line<'_, SmallFixedBuffers>>();
/// assert!(line_size <= 350, "Got {}", line_size);
///
/// // the explicit type for a `GCode` backed by `SmallFixedBuffers`
/// type SmallBufferGCode<'a> = GCode<<SmallFixedBuffers as Buffers<'a>>::Arguments>;
///
/// let gcode_size = std::mem::size_of::<SmallBufferGCode<'_>>(); 
/// assert!(gcode_size  <= 200, "Got {}", gcode_size);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SmallFixedBuffers {}

impl<'input> Buffers<'input> for SmallFixedBuffers {
    type Arguments = ArrayVec<[Word; 5]>;
    type Commands = ArrayVec<[GCode<Self::Arguments>; 1]>;
    type Comments = ArrayVec<[Comment<'input>; 1]>;
}

/// A [`Buffers`] implementation which uses [`std::vec::Vec`] for storing items.
///
/// In terms of memory usage, this has the potential to use a lot less overall 
/// than something like [`SmallFixedBuffers`] because we've traded deterministic
/// memory usage for only allocating memory when it is required.
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

/// An error returned when [`Buffer::try_push()`] fails. 
///
/// When a [`Buffer`] can't add an item, it will use [`CapacityError`] to pass 
/// the original item back to the caller.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CapacityError<T>(pub T);

impl<T: Debug> Display for CapacityError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "insufficient capacity")
    }
}

#[cfg(feature = "std")]
impl<T: Debug> std::error::Error for CapacityError<T> {}
