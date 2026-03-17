//! Diagnostics for alloc-based parsing; returned by [`parse`](crate::parse) when errors occur.
//!
//! The parser reports recoverable issues via the [`Diagnostics`](crate::core::Diagnostics) trait;
//! this module provides a concrete implementation that collects them.
#![allow(missing_docs)]

use alloc::{string::String, string::ToString, vec::Vec};
use core::fmt::{self, Display, Formatter};

use crate::core::{Span, TokenType};

/// A single recoverable parse issue with a [`DiagnosticKind`] and source [`Span`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Span,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Diagnostic {
            kind,
            span: Span { line, .. },
        } = self;
        let line = line + 1;

        write!(f, "{kind} on line {line}")
    }
}

/// Category of parse diagnostic emitted during recovery.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DiagnosticKind {
    /// text the parser could not interpret (e.g. invalid token).
    UnknownContent { text: String },
    /// the parser expected one of `expected` token types but found `actual`.
    Unexpected {
        actual: String,
        expected: Vec<TokenType>,
    },
}

impl Display for DiagnosticKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::UnknownContent { text } => {
                write!(f, "Unknown content: {}", text)
            },
            DiagnosticKind::Unexpected { actual, expected } => {
                let expected = expected
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Unexpected: {actual} (expected: {expected})")
            },
        }
    }
}

/// Collection of [`Diagnostic`]s produced by a parse.
///
/// Returned by [`parse`](crate::parse) in `Err` when any diagnostic was emitted.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Creates an empty collection.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns true if no diagnostics were collected.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes self and returns the inner vector of diagnostics.
    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.0
    }

    /// Iterates over the collected diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter()
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::Diagnostics for Diagnostics {
    fn emit_unknown_content(&mut self, text: &str, span: Span) {
        self.0.push(Diagnostic {
            kind: DiagnosticKind::UnknownContent {
                text: text.to_string(),
            },
            span,
        });
    }

    fn emit_unexpected(
        &mut self,
        actual: &str,
        expected: &[TokenType],
        span: Span,
    ) {
        self.0.push(Diagnostic {
            kind: DiagnosticKind::Unexpected {
                actual: actual.to_string(),
                expected: expected.to_vec(),
            },
            span,
        });
    }
}
