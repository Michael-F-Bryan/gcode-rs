use crate::Span;

#[cfg(feature = "std")]
use std::borrow::Cow;

/// A comment.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Comment<'input> {
    /// The comment itself.
    pub value: &'input str,
    /// Where the comment is located in the original string.
    pub span: Span,
}

/// An owned version of [`Comment`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg(feature = "std")]
pub struct OwnedComment {
    /// The comment itself.
    pub value: Cow<'static, str>,
    /// Where the comment is located in the original string.
    pub span: Span,
}

#[cfg(feature = "std")]
impl OwnedComment {
    pub fn new<S>(value: S, span: Span) -> OwnedComment
    where
        S: Into<Cow<'static, str>>,
    {
        OwnedComment {
            value: value.into(),
            span,
        }
    }
}

#[cfg(feature = "std")]
impl From<Comment<'static>> for OwnedComment {
    fn from(other: Comment<'static>) -> OwnedComment {
        OwnedComment::new(other.value, other.span)
    }
}
