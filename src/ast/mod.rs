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
