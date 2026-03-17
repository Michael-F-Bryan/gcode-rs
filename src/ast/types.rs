use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::core::{Number, Span};

#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Program {
    pub blocks: Vec<Block>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

impl core::str::FromStr for Program {
    type Err = crate::ast::Diagnostics;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::ast::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Block {
    pub line_number: Option<Number>,
    pub comments: Vec<Comment>,
    pub codes: Vec<Code>,
    /// Modal bare word addresses (e.g. `X5.0`, `S12000`) at block level without a G/M/T prefix.
    pub word_addresses: Vec<WordAddress>,
    pub span: Span,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut need_space = false;
        if let Some(n) = self.line_number {
            write!(f, "N{}", n)?;
            need_space = true;
        }
        for c in &self.comments {
            if need_space {
                write!(f, " ")?;
            }
            write!(f, "{}", c)?;
            need_space = true;
        }
        for code in &self.codes {
            if need_space {
                write!(f, " ")?;
            }
            write!(f, "{}", code)?;
            need_space = true;
        }
        for w in &self.word_addresses {
            if need_space {
                write!(f, " ")?;
            }
            write!(f, "{}", w)?;
            need_space = true;
        }
        writeln!(f)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct WordAddress {
    pub letter: char,
    pub value: Value,
    pub span: Span,
}

impl fmt::Display for WordAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.value)
    }
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

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CommentKind::Semicolon => write!(f, ";{}", self.value),
            CommentKind::Parentheses => write!(f, "({}", self.value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Code {
    General(GeneralCode),
    Miscellaneous(MiscellaneousCode),
    ToolChange(ToolChangeCode),
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Code::General(g) => write!(f, "{}", g),
            Code::Miscellaneous(m) => write!(f, "{}", m),
            Code::ToolChange(t) => write!(f, "{}", t),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct GeneralCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

impl fmt::Display for GeneralCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "G{}", self.number)?;
        for arg in &self.args {
            write!(f, "{}", arg)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct MiscellaneousCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

impl fmt::Display for MiscellaneousCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "M{}", self.number)?;
        for arg in &self.args {
            write!(f, "{}", arg)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ToolChangeCode {
    pub number: Number,
    pub span: Span,
    pub args: Vec<Argument>,
}

impl fmt::Display for ToolChangeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.number)?;
        for arg in &self.args {
            write!(f, "{}", arg)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Argument {
    pub letter: char,
    pub value: Value,
    pub span: Span,
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {}{}", self.letter, self.value)
    }
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Literal(n) => write!(f, "{}", n),
            Value::Variable(s) => write!(f, "#{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert two programs are semantically equal (ignoring spans).
    fn programs_semantic_eq(a: &Program, b: &Program) {
        let Program { blocks: a_blocks } = a;
        let Program { blocks: b_blocks } = b;
        assert_eq!(a_blocks.len(), b_blocks.len(), "block count mismatch");
        for (a_block, b_block) in a_blocks.iter().zip(b_blocks.iter()) {
            block_semantic_eq(a_block, b_block);
        }
    }

    fn block_semantic_eq(a: &Block, b: &Block) {
        let Block {
            line_number: a_line_number,
            comments: a_comments,
            codes: a_codes,
            word_addresses: a_word_addresses,
            span: _,
        } = a;
        let Block {
            line_number: b_line_number,
            comments: b_comments,
            codes: b_codes,
            word_addresses: b_word_addresses,
            span: _,
        } = b;
        assert_eq!(a_line_number, b_line_number);
        comments_semantic_eq(a_comments, b_comments);
        codes_semantic_eq(a_codes, b_codes);
        word_addresses_semantic_eq(a_word_addresses, b_word_addresses);
    }

    fn comments_semantic_eq(a: &[Comment], b: &[Comment]) {
        assert_eq!(a.len(), b.len(), "comment count mismatch");
        for (a_c, b_c) in a.iter().zip(b.iter()) {
            comment_semantic_eq(a_c, b_c);
        }
    }

    fn comment_semantic_eq(a: &Comment, b: &Comment) {
        let Comment {
            value: a_value,
            kind: a_kind,
            span: _,
        } = a;
        let Comment {
            value: b_value,
            kind: b_kind,
            span: _,
        } = b;
        assert_eq!(a_kind, b_kind);
        assert_eq!(a_value, b_value);
    }

    fn codes_semantic_eq(a: &[Code], b: &[Code]) {
        assert_eq!(a.len(), b.len(), "code count mismatch");
        for (a_c, b_c) in a.iter().zip(b.iter()) {
            code_semantic_eq(a_c, b_c);
        }
    }

    fn code_semantic_eq(a: &Code, b: &Code) {
        match (a, b) {
            (Code::General(ga), Code::General(gb)) => {
                general_code_semantic_eq(ga, gb)
            },
            (Code::Miscellaneous(ma), Code::Miscellaneous(mb)) => {
                misc_code_semantic_eq(ma, mb);
            },
            (Code::ToolChange(ta), Code::ToolChange(tb)) => {
                tool_change_code_semantic_eq(ta, tb);
            },
            _ => panic!("code variant mismatch: {:?} vs {:?}", a, b),
        }
    }

    fn general_code_semantic_eq(a: &GeneralCode, b: &GeneralCode) {
        let GeneralCode {
            number: a_number,
            span: _,
            args: a_args,
        } = a;
        let GeneralCode {
            number: b_number,
            span: _,
            args: b_args,
        } = b;
        assert_eq!(a_number, b_number);
        arguments_semantic_eq(a_args, b_args);
    }

    fn misc_code_semantic_eq(a: &MiscellaneousCode, b: &MiscellaneousCode) {
        let MiscellaneousCode {
            number: a_number,
            span: _,
            args: a_args,
        } = a;
        let MiscellaneousCode {
            number: b_number,
            span: _,
            args: b_args,
        } = b;
        assert_eq!(a_number, b_number);
        arguments_semantic_eq(a_args, b_args);
    }

    fn tool_change_code_semantic_eq(a: &ToolChangeCode, b: &ToolChangeCode) {
        let ToolChangeCode {
            number: a_number,
            span: _,
            args: a_args,
        } = a;
        let ToolChangeCode {
            number: b_number,
            span: _,
            args: b_args,
        } = b;
        assert_eq!(a_number, b_number);
        arguments_semantic_eq(a_args, b_args);
    }

    fn arguments_semantic_eq(a: &[Argument], b: &[Argument]) {
        assert_eq!(a.len(), b.len(), "argument count mismatch");
        for (a_arg, b_arg) in a.iter().zip(b.iter()) {
            argument_semantic_eq(a_arg, b_arg);
        }
    }

    fn argument_semantic_eq(a: &Argument, b: &Argument) {
        let Argument {
            letter: a_letter,
            value: a_value,
            span: _,
        } = a;
        let Argument {
            letter: b_letter,
            value: b_value,
            span: _,
        } = b;
        assert_eq!(a_letter, b_letter);
        value_semantic_eq(a_value, b_value);
    }

    fn word_addresses_semantic_eq(a: &[WordAddress], b: &[WordAddress]) {
        assert_eq!(a.len(), b.len(), "word address count mismatch");
        for (a_w, b_w) in a.iter().zip(b.iter()) {
            word_address_semantic_eq(a_w, b_w);
        }
    }

    fn word_address_semantic_eq(a: &WordAddress, b: &WordAddress) {
        let WordAddress {
            letter: a_letter,
            value: a_value,
            span: _,
        } = a;
        let WordAddress {
            letter: b_letter,
            value: b_value,
            span: _,
        } = b;
        assert_eq!(a_letter, b_letter);
        value_semantic_eq(a_value, b_value);
    }

    fn value_semantic_eq(a: &Value, b: &Value) {
        match (a, b) {
            (Value::Literal(la), Value::Literal(lb)) => {
                assert!(
                    (la - lb).abs() < 1e-6,
                    "literal value mismatch: {} vs {}",
                    la,
                    lb
                );
            },
            (Value::Variable(va), Value::Variable(vb)) => assert_eq!(va, vb),
            _ => panic!("value variant mismatch: {:?} vs {:?}", a, b),
        }
    }

    macro_rules! roundtrip_test {
        (
            $( $(#[$attr:meta])* $name:ident => $src:expr),* $(,)?
        ) => {
            $(
                $(#[$attr])*
                #[test]
                fn $name() {
                    let original = crate::parse($src).unwrap();
                    let serialized = original.to_string();
                    let roundtripped = crate::parse(&serialized).unwrap();
                    programs_semantic_eq(&original, &roundtripped);
                }
            )*
        };
    }

    roundtrip_test! {
        roundtrip_single_g_code => "G0\n",
        roundtrip_g_code_with_args => "G0 X1 Y2\n",
        roundtrip_line_number_and_paren_comment => "N10 (metric) G21\n",
        roundtrip_semicolon_comment => "; comment\n",
        roundtrip_word_addresses => "X5.0 Y-3.0\n",
        roundtrip_m_code => "M3 S1000\n",
        roundtrip_t_code => "T1\n",
        roundtrip_empty_block => "\n",
        roundtrip_multiple_blocks => "G0 X0\nM3 S1000\nT1\n",
        roundtrip_mixed_content => "N10 (setup) G21 G90 X0 Y0 M3 S1000\n",
        roundtrip_negative_and_decimal => "G0 X-3.5 Y0.1\n",
        roundtrip_whitespace_variations => "  G0   X1   Y2  \n",
        roundtrip_code_with_minor_number => "G91.1\n",
        roundtrip_empty_program => "",
        roundtrip_two_comments_same_line => "(first) ; second\n",
        #[ignore = "parser does not yet parse # variable syntax"]
        roundtrip_variable => "G0 X#1\n",
    }

    /// Parser does not yet support # variable syntax, so roundtrip cannot be tested.
    /// This test verifies that Value::Variable displays as "#" + value.
    #[test]
    fn display_variable_formats_with_hash_prefix() {
        let v = Value::Variable("1".into());
        assert_eq!(v.to_string(), "#1");
        let v = Value::Variable("var".into());
        assert_eq!(v.to_string(), "#var");
    }
}
