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

    #[test]
    fn ast_parse_captures_m_and_t_codes() {
        let program = crate::parse("G0 X0\nM3 S1000\nT1\n").unwrap();
        assert_eq!(program.blocks.len(), 3);

        assert_eq!(program.blocks[0].codes.len(), 1);
        assert!(matches!(
            &program.blocks[0].codes[0],
            crate::ast::Code::General(_)
        ));

        assert_eq!(program.blocks[1].codes.len(), 1);
        assert!(matches!(
            &program.blocks[1].codes[0],
            crate::ast::Code::Miscellaneous(m) if m.number.major == 3
        ));

        assert_eq!(program.blocks[2].codes.len(), 1);
        assert!(matches!(
            &program.blocks[2].codes[0],
            crate::ast::Code::ToolChange(t) if t.number.major == 1
        ));
    }
}
