use core::iter::Peekable;
use lexer::{Lexer, Word};
use types::{Gcode, Mnemonic};
use number::Number;

pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src)
}

/// A gcode parser which is extremely permissive in what input it will accept.
#[derive(Debug, Clone)]
struct Parser<'input> {
    lexer: Peekable<Lexer<'input>>,
}

impl<'input> Parser<'input> {
    pub fn new(src: &'input str) -> Parser<'input> {
        Parser {
            lexer: Lexer::new(src).peekable(),
        }
    }

    fn is_finished(&mut self) -> bool {
        self.peek().is_none()
    }

    fn peek(&mut self) -> Option<&Word> {
        self.lexer.peek()
    }
}

impl<'input> Iterator for Parser<'input> {
    type Item = Gcode;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.lexer.next()?;

        match word.letter {
            'O' => Some(Gcode::new(Mnemonic::ProgramNumber, word.number.convert(), word.span)),
            'T' => Some(Gcode::new(Mnemonic::ToolChange, word.number.convert(), word.span)),
            'M' => Some(Gcode::new(Mnemonic::MachineRoutine, word.number.convert(), word.span)),
            'G' => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Span;

    macro_rules! parse_test {
        ($name:ident, $src:expr => $should_be:expr) => (
            #[test]
            fn $name() {
                let src = $src;
                let should_be = $should_be;

                let mut parser = Parser::new(src);

                assert!(!parser.is_finished());
                let got = parser.next().unwrap();
                assert!(parser.is_finished());

                assert_eq!(got, should_be);
            }
        )
    }

    parse_test!(parse_a_program_number, "O123" => Gcode {
            mnemonic: Mnemonic::ProgramNumber,
            number: Number::from(123),
            span: Span::new(0, 4, 0),
        });

    parse_test!(parse_a_tool_change, "T6" => Gcode {
            mnemonic: Mnemonic::ToolChange,
            number: Number::from(6),
            span: Span::new(0, 2, 0),
        });

    parse_test!(parse_a_machine_code, "M30" => Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: Number::from(30),
            span: Span::new(0, 3, 0),
        });
}
