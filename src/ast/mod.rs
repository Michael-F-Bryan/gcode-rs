//! Parse G-code into an AST.

#![allow(missing_docs)]

mod diags;
mod types;
mod visitor;

pub use self::{
    diags::{Diagnostic, Diagnostics},
    types::*,
    visitor::AstBuilder,
};

pub fn parse(src: &str) -> Result<Program, Diagnostics> {
    let mut visitor = AstBuilder::new();
    crate::core::parse(src, &mut visitor);
    visitor.finish()
}

#[cfg(test)]
mod tests {

    #[test]
    fn alloc_parse_captures_word_addresses() {
        let program = crate::parse("X5.0 Y-3.0\n").unwrap();
        assert_eq!(program.blocks.len(), 1);
        assert_eq!(program.blocks[0].word_addresses.len(), 2);
        assert_eq!(program.blocks[0].word_addresses[0].letter, 'X');
        assert!(matches!(
            program.blocks[0].word_addresses[0].value,
            crate::ast::Value::Literal(n) if (n - 5.0).abs() < 1e-6
        ));
        assert_eq!(program.blocks[0].word_addresses[1].letter, 'Y');
        assert!(matches!(
            program.blocks[0].word_addresses[1].value,
            crate::ast::Value::Literal(n) if (n - (-3.0)).abs() < 1e-6
        ));
    }
}
