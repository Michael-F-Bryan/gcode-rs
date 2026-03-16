use alloc::vec::Vec;

use crate::{
    ast::{
        Argument, Block, Code, Comment, CommentKind, Diagnostics, GeneralCode,
        Program,
    },
    core::{
        ControlFlow, Diagnostics as _, HasDiagnostics, Number, ProgramVisitor,
        Span, TokenType,
    },
};

#[derive(Debug)]
pub struct AstBuilder {
    blocks: Vec<Block>,
    diagnostics: Diagnostics,
}

impl AstBuilder {
    pub const fn new() -> Self {
        Self {
            blocks: Vec::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    pub fn finish(self) -> Result<Program, Diagnostics> {
        let AstBuilder {
            blocks,
            diagnostics,
        } = self;
        if diagnostics.is_empty() {
            Ok(Program { blocks })
        } else {
            Err(diagnostics)
        }
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HasDiagnostics for AstBuilder {
    fn diagnostics(&mut self) -> &mut dyn crate::core::Diagnostics {
        &mut self.diagnostics
    }
}

impl ProgramVisitor for AstBuilder {
    fn start_block(
        &mut self,
    ) -> ControlFlow<impl crate::core::BlockVisitor + '_> {
        ControlFlow::Continue(BlockBuilder::new(
            &mut self.blocks,
            &mut self.diagnostics,
        ))
    }
}

#[derive(Debug)]
struct BlockBuilder<'a> {
    blocks: &'a mut Vec<Block>,
    diags: &'a mut Diagnostics,
    comments: Vec<Comment>,
    codes: Vec<Code>,
    line_number: Option<Number>,
}

impl<'a> BlockBuilder<'a> {
    fn new(blocks: &'a mut Vec<Block>, diags: &'a mut Diagnostics) -> Self {
        Self {
            blocks,
            diags,
            comments: Vec::new(),
            codes: Vec::new(),
            line_number: None,
        }
    }
}

impl crate::core::BlockVisitor for BlockBuilder<'_> {
    fn line_number(&mut self, n: Number, _: Span) {
        self.line_number = Some(n);
    }

    fn comment(&mut self, value: &str, span: Span) {
        let (kind, value) = if let Some(value) = value.strip_prefix(';') {
            (CommentKind::Semicolon, value)
        } else if let Some(value) = value.strip_prefix('(') {
            (CommentKind::Parentheses, value)
        } else {
            return self.diags.emit_unexpected(value, &[TokenType::Comment], span);
        };

        self.comments.push(Comment {
            value: value.into(),
            span,
            kind,
        });
    }

    fn start_general_code(
        &mut self,
        number: Number,
    ) -> ControlFlow<impl crate::core::CommandVisitor + '_> {
        let v = CodeBuilder {
            diags: self.diags,
            number,
            codes: &mut self.codes,
            constructor: |number, args, span| {
                Code::General(GeneralCode { number, span, args })
            },
            args: Vec::new(),
        };
        core::ops::ControlFlow::Continue(v)
    }

    fn end_line(self, span: Span) {
        let block = Block {
            line_number: self.line_number,
            comments: self.comments,
            codes: self.codes,
            span,
        };
        self.blocks.push(block);
    }
}

impl HasDiagnostics for BlockBuilder<'_> {
    fn diagnostics(&mut self) -> &mut dyn crate::core::Diagnostics {
        self.diags
    }
}

struct CodeBuilder<'a, F> {
    codes: &'a mut Vec<Code>,
    diags: &'a mut Diagnostics,
    constructor: F,
    args: Vec<Argument>,
    number: Number,
}

impl<F: FnOnce(Number, Vec<Argument>, Span) -> Code> crate::core::CommandVisitor
    for CodeBuilder<'_, F>
{
    fn argument(
        &mut self,
        letter: char,
        value: crate::core::Value<'_>,
        span: Span,
    ) {
        self.args.push(Argument {
            letter,
            value: value.into(),
            span,
        });
    }

    fn end_command(self, span: Span) {
        let code = (self.constructor)(self.number, self.args, span);
        self.codes.push(code);
    }
}

impl<F> HasDiagnostics for CodeBuilder<'_, F> {
    fn diagnostics(&mut self) -> &mut dyn crate::core::Diagnostics {
        self.diags
    }
}
