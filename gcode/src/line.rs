use crate::{
    buffers::{self, Buffer, Buffers, CapacityError, DefaultBuffers},
    Comment, GCode, Span, Word,
};
use core::fmt::{self, Debug, Formatter};

/// A single line, possibly containing some [`Comment`]s or [`GCode`]s.
#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
pub struct Line<'input, B: Buffers<'input> = DefaultBuffers> {
    gcodes: B::Commands,
    comments: B::Comments,
    line_number: Option<Word>,
    span: Span,
}

impl<'input, B> Debug for Line<'input, B>
where
    B: Buffers<'input>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // explicitly implement Debug because the normal derive is too strict
        let Line {
            gcodes,
            comments,
            line_number,
            span,
        } = self;

        f.debug_struct("Line")
            .field("gcodes", &buffers::debug(gcodes))
            .field("comments", &buffers::debug(comments))
            .field("line_number", line_number)
            .field("span", span)
            .finish()
    }
}

impl<'input, B> Default for Line<'input, B>
where
    B: Buffers<'input>,
    B::Commands: Default,
    B::Comments: Default,
{
    fn default() -> Line<'input, B> {
        Line {
            gcodes: B::Commands::default(),
            comments: B::Comments::default(),
            line_number: None,
            span: Span::default(),
        }
    }
}

impl<'input, B: Buffers<'input>> Line<'input, B> {
    /// All [`GCode`]s in this line.
    pub fn gcodes(&self) -> &[GCode<B::Arguments>] { self.gcodes.as_slice() }

    /// All [`Comment`]s in this line.
    pub fn comments(&self) -> &[Comment<'input>] { self.comments.as_slice() }

    /// Try to add another [`GCode`] to the line.
    pub fn push_gcode(
        &mut self,
        gcode: GCode<B::Arguments>,
    ) -> Result<(), CapacityError<GCode<B::Arguments>>> {
        // Note: We need to make sure a failed push doesn't change our span
        let span = self.span.merge(gcode.span());
        self.gcodes.try_push(gcode)?;
        self.span = span;

        Ok(())
    }

    /// Try to add a [`Comment`] to the line.
    pub fn push_comment(
        &mut self,
        comment: Comment<'input>,
    ) -> Result<(), CapacityError<Comment<'input>>> {
        let span = self.span.merge(comment.span);
        self.comments.try_push(comment)?;
        self.span = span;
        Ok(())
    }

    /// Does the [`Line`] contain anything at all?
    pub fn is_empty(&self) -> bool {
        self.gcodes.as_slice().is_empty()
            && self.comments.as_slice().is_empty()
            && self.line_number().is_none()
    }

    /// Try to get the line number, if there was one.
    pub fn line_number(&self) -> Option<Word> { self.line_number }

    /// Set the [`Line::line_number()`].
    pub fn set_line_number<W: Into<Option<Word>>>(&mut self, line_number: W) {
        match line_number.into() {
            Some(n) => {
                self.span = self.span.merge(n.span);
                self.line_number = Some(n);
            },
            None => self.line_number = None,
        }
    }

    /// Get the [`Line`]'s position in its source text.
    pub fn span(&self) -> Span { self.span }

    pub(crate) fn into_gcodes(self) -> B::Commands { self.gcodes }
}
