use alloc::{string::String, vec::Vec};

use crate::core::{Number, Span};

#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Program {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Block {
    pub line_number: Option<Number>,
    pub comments: Vec<Comment>,
    pub codes: Vec<Code>,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum CommentKind {
    Semicolon,
    Parentheses,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Comment {
    pub value: String,
    pub kind: CommentKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Code {
    General(GeneralCode),
    Miscellaneous(MiscellaneousCode),
    ToolChange(ToolChangeCode),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct GeneralCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct MiscellaneousCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ToolChangeCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Argument {
    pub letter: char,
    pub value: Value,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Value {
    Literal(f32),
    Variable(String),
}

impl From<crate::core::Value<'_>> for Value {
    fn from(value: crate::core::Value<'_>) -> Self {
        match value {
            crate::core::Value::Literal(l) => Self::Literal(l),
            crate::core::Value::Variable(v) => Self::Variable(v.into()),
        }
    }
}
