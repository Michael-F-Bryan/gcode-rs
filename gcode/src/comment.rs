use crate::Span;

/// A comment.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde-1",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
pub struct Comment<'input> {
    /// The comment itself.
    pub value: &'input str,
    /// Where the comment is located in the original string.
    pub span: Span,
}
