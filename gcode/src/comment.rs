use crate::Span;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Comment<'input> {
    pub value: &'input str,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg(feature = "std")]
pub struct OwnedComment {
    pub value: Cow<'static, str>,
    pub span: Span,
}

#[cfg(feature = "std")]
impl OwnedComment {
    pub fn new<S>(value: S, span: Span) -> OwnedComment 
    where S: Into<Cow<'static, str>>{
        OwnedComment {
            value: value.into(),
            span: span,
        }
    }
}

#[cfg(feature = "std")]
impl From<Comment<'static>> for OwnedComment {
    fn from(other: Comment<'static>) -> OwnedComment {
        OwnedComment::new(other.value, other.span)
    }
}
