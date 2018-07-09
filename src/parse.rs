use lexer::Lexer;
use types::Gcode;

pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src)
}

/// A gcode parser which is extremely permissive in what input it will accept.
#[derive(Debug, Copy, Clone, PartialEq)]
struct Parser<'input> {
    lexer: Lexer<'input>,
}

impl<'input> Parser<'input> {
    pub fn new(src: &'input str) -> Parser<'input> {
        Parser {
            lexer: Lexer::new(src),
        }
    }
}

impl<'input> Iterator for Parser<'input> {
    type Item = Gcode;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
