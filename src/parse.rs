use core::iter::Peekable;
use lexer::Lexer;
use types::{Gcode, Mnemonic, Word};

/// Parse a string of text into a stream of `Gcode`s.
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

    fn parse_machine(&mut self, m: Word) -> Gcode {
        Gcode::new(
                Mnemonic::MachineRoutine,
                m.number.convert(),
                m.span,
            )
    }

    fn parse_g(&mut self, m: Word) -> Gcode {
        unimplemented!()
    }

    fn is_arg(&self, c: char) -> bool {
        // we just assume all letters except mnemonics are argument material
        match c.to_ascii_uppercase() {
            'O' | 'M' | 'T' | 'G' => false,
            other if other.is_ascii_alphabetic() => true,
            _ => false,
        }
    }
}

impl<'input> Iterator for Parser<'input> {
    type Item = Gcode;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.lexer.next()?;

        match word.letter {
            'O' => Some(Gcode::new(
                Mnemonic::ProgramNumber,
                word.number.convert(),
                word.span,
            )),
            'T' => Some(Gcode::new(
                Mnemonic::ToolChange,
                word.number.convert(),
                word.span,
            )),
            'M' => Some(self.parse_machine(word)),
            'G' => Some(self.parse_g(word)),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Span;
    use number::Number;

    macro_rules! parse_test {
        ($name:ident, $src:expr => $should_be:expr) => {
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
        };
    }

    parse_test!(parse_a_program_number, "O123" => Gcode {
            mnemonic: Mnemonic::ProgramNumber,
            number: Number::from(123),
            span: Span::new(0, 4, 0),
            ..Default::default()
        });

    parse_test!(parse_a_tool_change, "T6" => Gcode {
            mnemonic: Mnemonic::ToolChange,
            number: Number::from(6),
            span: Span::new(0, 2, 0),
            ..Default::default()
        });

    parse_test!(parse_a_machine_code, "M30" => Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: Number::from(30),
            span: Span::new(0, 3, 0),
            ..Default::default()
        });
}
