use core::iter::Peekable;

use errors::*;


/// A parser which takes a stream of characters and parses them as gcode
/// instructions.
///
/// The grammar currently being used is roughly as follows:
///
/// TODO: Add the language grammmar and use that to direct parser development
///
/// ```text
///
/// ```
pub struct Parser<I>
    where I: Iterator<Item = char>
{
    stream: Peekable<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item = char>
{
    pub fn new(stream: I) -> Parser<I> {
        Parser { stream: stream.peekable() }
    }
}
