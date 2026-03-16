use alloc::{string::String, string::ToString, vec::Vec};
use core::fmt::{self, Display, Formatter};

use crate::core::{Span, TokenType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticKind {
    UnknownContent {
        text: String,
    },
    Unexpected {
        actual: String,
        expected: Vec<String>,
    },
}

impl Display for DiagnosticKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::UnknownContent { text } => {
                write!(f, "Unknown content: {}", text)
            },
            DiagnosticKind::Unexpected { actual, expected } => write!(
                f,
                "Unexpected: {} (expected: {})",
                actual,
                expected.join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.0
    }

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
                expected: expected.iter().map(ToString::to_string).collect(),
            },
            span,
        });
    }
}
